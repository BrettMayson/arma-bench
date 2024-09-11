# arma-bench

because I want to see how fast HEMTT optimizations go.

```sh
docker volume create arma_servers
docker run -d 
    -e STEAM_USER=
    -e STEAM_PASS=
    -v arma_servers:/opt/servers/
    -p 7562:7562
    ghcr.io/brettmayson/arma-bench:latest
```
