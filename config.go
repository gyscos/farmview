package main

import (
	"io/ioutil"
	"log"
	"os"

	"github.com/BurntSushi/toml"
)

func ReadConfig(filename string) (Config, error) {
	var config Config
	config.filename = filename

	log.Println("Reading configuration from", filename)
	data, err := ioutil.ReadFile(filename)
	if err != nil {
		return config, err
	}

	if err = toml.Unmarshal(data, &config); err != nil {
		return config, err
	}
	return config, nil
}

type Config struct {
	Http    HttpConfig
	Hosts   []HostConfig
	Default AuthConfig

	filename string
}

type HttpConfig struct {
	Port int
}

type HostConfig struct {
	Name    string
	Address string
	Disks   []string
	Auth    AuthConfig
}

type AuthConfig struct {
	Login    string
	Password string
	Keypair  string
}

func (c *Config) GetAuth(i int) AuthConfig {
	auth := c.Hosts[i].Auth
	if auth.Login != "" {
		return auth
	}

	return c.Default
}

func (c *Config) WriteConfig() error {
	f, err := os.Create(c.filename)
	if err != nil {
		return err
	}
	defer f.Close()

	return toml.NewEncoder(f).Encode(c)
}
