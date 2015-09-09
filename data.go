package main

// This is what is given to the HTTP server to prints beautiful information
// This is actually forwarded to the template engine.
type Data struct {
	Hosts []HostData

	UpdateTime string
}

type HostData struct {
	Name       string
	Online     bool
	Responsive bool
	Ping       string
	NCpu       int
	Load       [3]float64
	RamUsage   MemoryData
	DiskUsage  []DiskData
}

type MemoryData struct {
	TotalH      string
	TotalK      uint64
	UsedK       uint64
	PercentUsed int
}

type DiskData struct {
	Name        string
	Mount       string
	TotalH      string
	TotalK      uint64
	UsedK       uint64
	PercentUsed int
}
