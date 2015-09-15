package main

import (
	"flag"
	"log"
	"os"
)

// Default configuration directory, can be set at compile time
var conf_dir string

// Default data directory, can be set at compile time (see build.sh)
var data_dir string

func exists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

func find_config() string {
	paths := []string{
		"~/.config/farmview",
		"~/.farmview",
		"/etc/farmview",
		conf_dir,
		"."}
	for _, path := range paths {
		if exists(path + "/config.toml") {
			return path + "/config.toml"
		}
	}
	log.Fatal("Could not find a config file!")
	return ""
}

func main() {

	var config_file string
	var dataDir string
	var listen_port int
	flag.StringVar(&dataDir, "d", "", "data directory override")
	flag.StringVar(&config_file, "c", "", "config file override")
	flag.IntVar(&listen_port, "p", 0, "port to listen to")

	flag.Parse()

	if config_file == "" {
		config_file = find_config()
	}

	config, err := ReadConfig(config_file)
	if err != nil {
		log.Println("Error reading config file:")
		log.Fatal(err)
	}

	if dataDir == "" {
		if data_dir != "" {
			dataDir = data_dir
		} else {
			dataDir = "."
		}
	}

	if listen_port == 0 {
		listen_port = config.Http.Port
	}

	// Two services: http server and data fetcher
	dc := make(chan Data, 1)
	fetcher := NewFetcher(config)
	go fetcher.Fetch_loop(dc)

	server := NewServer(dataDir, dc)
	server.Serve(listen_port)
}
