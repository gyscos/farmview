package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os/exec"
	"os/user"
	"strconv"
	"strings"
	"time"

	humanize "github.com/dustin/go-humanize"
	"golang.org/x/crypto/ssh"
)

type Fetcher struct {
	config Config
}

func NewFetcher(config Config) Fetcher {
	var fetcher Fetcher

	fetcher.config = config

	return fetcher
}

func (f *Fetcher) getSshAuthMethod(auth AuthConfig) (ssh.AuthMethod, error) {
	if auth.Keypair != "" {
		usr, _ := user.Current()
		homeDir := usr.HomeDir
		keyPath := strings.Replace(auth.Keypair, "~", homeDir, -1)
		bytes, err := ioutil.ReadFile(keyPath)
		if err != nil {
			return nil, err
		}

		signer, err := ssh.ParsePrivateKey(bytes)
		if err != nil {
			return nil, err
		}
		return ssh.PublicKeys(signer), nil
	} else {
		return ssh.Password(auth.Password), nil
	}
}

func (f *Fetcher) makeClient(i int) (*ssh.Client, error) {

	auth := f.config.GetAuth(i)

	authMethod, err := f.getSshAuthMethod(auth)
	if err != nil {
		return nil, err
	}

	sshConfig := &ssh.ClientConfig{
		User: auth.Login,
		Auth: []ssh.AuthMethod{authMethod},
	}
	client, err := ssh.Dial("tcp", fmt.Sprintf("%v:22", f.config.Hosts[i].Address), sshConfig)
	if err != nil {
		return nil, err
	}

	return client, nil
}

func (f *Fetcher) FetchLoop(dc chan<- Data) {
	// First fetch
	dc <- f.Fetch()

	log.Println("Ready!")

	// Then, every few minutes
	for _ = range time.Tick(1 * time.Minute) {
		dc <- f.Fetch()
	}
}

func getPing(host string) (float64, error) {
	nPings := 3
	bytes, err := exec.Command("ping", "-nc", strconv.Itoa(nPings), host).Output()
	if err != nil {
		return 0.0, err
	}

	sum := 0.0

	lines := strings.Split(string(bytes), "\n")

	nErrors := 0
	for i := 0; i < nPings; i++ {
		if i+1 >= len(lines) {
			nErrors += 1
			continue
		}
		fields := strings.Fields(lines[1+i])
		if len(fields) < 2 {
			nErrors += 1
			continue
		}
		n := len(fields) - 2
		tokens := strings.Split(fields[n], "=")
		if len(tokens) < 2 {
			nErrors += 1
			continue
		}
		ping, err := strconv.ParseFloat(tokens[1], 64)
		if err != nil {
			nErrors += 1
			continue
		}
		sum += ping
	}
	if nErrors == nPings {
		log.Println("Bad ping response:\n", string(bytes))
	}

	return sum / float64(nPings-nErrors), nil
}

func fillPing(hostData *HostData, address string) {
	ping, err := getPing(address)
	if err != nil {
		// log.Println("Could not get ping:", err)
	} else {
		hostData.Online = true
	}
	hostData.Ping = fmt.Sprintf("%.1f", ping)
}

func fillDetails(hostData *HostData, host *HostConfig, f *Fetcher, i int) {
	client, err := f.makeClient(i)
	if err != nil {
		log.Println("SSH connection failure:", err)
		return
	}
	defer client.Close()

	session, err := client.NewSession()
	if err != nil {
		log.Println("SSH session failure:", err)
		return
	}
	defer session.Close()

	bytes, err := session.Output("nproc && uptime && cat /proc/meminfo | head -n 4 && df -P")
	if err != nil {
		log.Println("SSH command failed:", err)
		return
	}
	parseResult(string(bytes), hostData, host.Disks)

	hostData.Responsive = true
}

func (f *Fetcher) makeHostData(i int) (HostData, error) {
	var hostData HostData

	host := &f.config.Hosts[i]
	hostData.Name = host.Name

	jc := make(chan struct{})
	go func() {
		fillPing(&hostData, host.Address)
		jc <- struct{}{}
	}()

	fillDetails(&hostData, host, f, i)

	// Wait for ping job to complete
	<-jc
	// log.Println("Host", host.Name, "ready")
	return hostData, nil
}

func getPercentUsed(used uint64, total uint64) int {

	return int((100*used + total/2) / total)
}

func parseResult(output string, data *HostData, diskNames []string) {
	var err error
	lines := strings.Split(output, "\n")
	lineId := 0

	data.NCpu, err = strconv.Atoi(lines[lineId])
	if err != nil {
		log.Println("Error parsing nproc:", err)
	}
	lineId++

	// Second line is from uptime
	uptimeLine := strings.Fields(lines[lineId])
	lineId++
	n := len(uptimeLine)

	for i := 0; i < 3; i++ {
		data.Load[i], err = strconv.ParseFloat(strings.TrimRight(uptimeLine[n-3+i], ","), 64)
		if err != nil {
			log.Println("Error parsing uptime:", err)
		}
	}
	data.CpuUsage = int(100*data.Load[0]) / data.NCpu

	data.RamUsage.TotalK, err = strconv.ParseUint(strings.Fields(lines[lineId])[1], 10, 64)
	if err != nil {
		log.Println("Error parsing total RAM:", err)
	}
	if strings.Contains(lines[lineId+2], "Available") {
		availableK, err := strconv.ParseUint(strings.Fields(lines[lineId+2])[1], 10, 64)
		if err != nil {
			log.Println("Error parsing available RAM:", err)
		}
		data.RamUsage.UsedK = data.RamUsage.TotalK - availableK
	} else {
		freeK, err := strconv.ParseUint(strings.Fields(lines[lineId+1])[1], 10, 64)
		if err != nil {
			log.Println("Error parsing free RAM:", err)
		}
		cacheK, err := strconv.ParseUint(strings.Fields(lines[lineId+3])[1], 10, 64)
		if err != nil {
			log.Println("Error parsing cached RAM:", err)
		}
		availableK := freeK + cacheK
		data.RamUsage.UsedK = data.RamUsage.TotalK - availableK

	}
	data.RamUsage.PercentUsed = getPercentUsed(data.RamUsage.UsedK, data.RamUsage.TotalK)
	data.RamUsage.TotalH = humanize.IBytes(data.RamUsage.TotalK * 1024)
	lineId += 4

	// Fourth and Fivth lines are useless.
	data.DiskUsage = make([]DiskData, len(diskNames))
	for i := 5; i < len(lines); i++ {
		line := strings.Fields(lines[i])
		if len(line) < 6 {
			continue
		}
		diskName := line[0]
		mountName := line[5]
		index := indexOf(diskName, diskNames)
		if index == -1 {
			index = indexOf(mountName, diskNames)
		}
		if index != -1 {
			totalK, err := strconv.ParseUint(line[1], 10, 64)
			if err != nil {
				continue
			}
			availK, err := strconv.ParseUint(line[3], 10, 64)
			if err != nil {
				continue
			}

			diskUsage := &data.DiskUsage[index]
			diskUsage.Name = diskName
			diskUsage.Mount = mountName
			diskUsage.TotalK = totalK
			diskUsage.UsedK = totalK - availK
			diskUsage.PercentUsed = getPercentUsed(diskUsage.UsedK, diskUsage.TotalK)
			diskUsage.TotalH = humanize.IBytes(diskUsage.TotalK * 1024)
		}
	}
}

func indexOf(value string, list []string) int {
	for i, entry := range list {
		if value == entry {
			return i
		}
	}
	for i, entry := range list {
		if strings.Contains(value, entry) {
			return i
		}
	}
	return -1
}

func (f *Fetcher) Fetch() Data {
	var data Data

	data.Hosts = make([]HostData, len(f.config.Hosts))

	count := 0
	wc := make(chan struct{})

	for i, _ := range f.config.Hosts {
		go func(i int) {
			hostData, err := f.makeHostData(i)
			if err != nil {
				// ???
				log.Println("Error making host data:", err)
			}
			data.Hosts[i] = hostData
			wc <- struct{}{}
		}(i)
		count++
	}

	for i := 0; i < count; i++ {
		<-wc
	}
	data.UpdateTime = time.Now().Format("2006-01-02 at 15:04")

	// log.Println("Fetched", data)

	return data
}
