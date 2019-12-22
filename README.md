# Twinkle

UDP-based, light key-value store.

[![Actions Status](https://github.com/kirisaki/twinkle/workflows/test-twinkle/badge.svg)](https://github.com/kirisaki/twinkle/actions)

[![dockeri.co](https://dockeri.co/image/kirisaki/twinkle)](https://hub.docker.com/r/kirisaki/twinkle)

## benchmark

```
twinkle:
0.176secs elapsed for 2000 requests
redis:
0.210secs elapsed for 2000 requests
```
[Benchmark code is here.](https://github.com/kirisaki/twinkle-bench)

## Usage

Use docker. Remenber to open UDP port.

```shell
$ docker run -d -p 3000:3000/udp kirisaki:twinkle
```

Docker repositry is https://hub.docker.com/repository/docker/kirisaki/twinkle

## Client libraly

- Rust https://github.com/kirisaki/twinkle-rust

## License

[BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause)
