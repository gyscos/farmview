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
