Dashboard console to see the status of multiple servers.

```
go get github.com/Gyscos/farmview
```

```
Usage of farmview:
  -c string
    	config file override
  -d string
    	data directory override
  -p int
    	port to listen to
```

The config.toml file specifies the farm configuration.


To package it, use these two scripts:

```
$ ./build.sh
```

To install (you may want to set the DESTDIR var first):

```
# ./install.sh
```

Or you could use the included Dockerfile:

```
# Build a docker image
./dockerify.sh

# Run the docker image
./run.sh path_to/config.toml 8123 # other options are directly passed to `docker run`
```
