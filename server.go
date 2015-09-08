package main

import (
	"fmt"
	"html/template"
	"log"
	"net/http"
)

type Server struct {
	CurrentData Data

	nodeList *template.Template
}

func NewServer(dataDir string, dc <-chan Data) *Server {
	var server Server

	go func() {
		for data := range dc {
			server.CurrentData = data
		}
	}()

	err := server.LoadTemplates(dataDir)
	if err != nil {
		log.Fatal(err)
	}

	// Now setup some handlers...
	http.HandleFunc("/index", server.ServeIndex)
	http.Handle("/", http.FileServer(http.Dir(dataDir+"/static")))

	return &server
}

func (s *Server) LoadTemplates(dataDir string) error {
	log.Println("Loading", dataDir+"/templates/node_list.html")
	nodeList, err := template.ParseFiles(dataDir + "/templates/node_list.html")
	if err != nil {
		return err
	}

	s.nodeList = nodeList

	return nil
}

func (s *Server) ServeIndex(w http.ResponseWriter, r *http.Request) {

	s.nodeList.Execute(w, s.CurrentData)
}

func (s *Server) Serve(port int) {
	log.Println("Now listening on port", port)
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%v", port), nil))
}
