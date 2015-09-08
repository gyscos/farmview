package main

// This is what is given to the HTTP server to prints beautiful information
// This is actually forwarded to the template engine.
type Data struct {
	Hosts []HostData

	UpdateTime string
}

type HostData struct {
	Name      string
	Online    bool
	Ping      float32
	NCpu      int
	Load      [3]float64
	RamUsage  MemoryData
	DiskUsage []DiskData
}

type MemoryData struct {
	TotalK      int64
	UsedK       int64
	PercentUsed int
}

type DiskData struct {
	Name        string
	Mount       string
	TotalK      int64
	UsedK       int64
	PercentUsed int
}
