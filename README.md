# Twinkle

UDP-based, light key-value store.

## Usage

Use docker. Remenber to open UDP port.

```shell
$ docker run -d -p 3000:3000/udp kirisaki:twinkle
```

Docker repositry is https://hub.docker.com/repository/docker/kirisaki/twinkle

## Client libraly

- Rust https://github.com/kirisaki/twinkle-rust

## Protocol

The protcol of twinkle is quiet simple. Only send binary messages expressed following.
Its byte-order is big endian.

### Ping

#### Request
```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x01  |                UUID     
 |        |
 +--------+--------+--------+------...
          |
          |
 ...------+
```

#### Response
```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x01  |                UUID     
 |        |
 +--------+--------+--------+------...
          |
          |
 ...------+
```

### Get

#### Reuest

```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x02  |                UUID     
 |        |
 +--------+--------+--------+------...
          |       Key       |
          |      Length     | key octets ...
 ...------+-----------------+-----...
```

#### Response(key found)

```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x01  |                UUID     
 |        |
 +--------+--------+--------+------...
          |
          | value octets
 ...------+--------+------...
```

#### Response(key not found)
```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x02  |                UUID     
 |        |
 +--------+--------+--------+------...
          |
          |
 ...------+
```

### Set

#### Request
```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x03  |                UUID     
 |        |
 +--------+--------+--------+------...
          |       Key       |
          |      Length     | key octets ...
 ...------+-----------------+-----...
          |
          | value octets ...
 ...------+--------------...

```

#### Response
```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x01  |                UUID     
 |        |
 +--------+--------+--------+------...
          |
          |
 ...------+
```

### Unset

#### Request
```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x04  |                UUID     
 |        |
 +--------+--------+--------+------...
          |       Key       |
          |      Length     | key octets ...
 ...------+-----------------+-----...
```

#### Response
```
  0      7 8     15 16    23 24    31
 +--------+--------+--------+--------+
 |  0x01  |                UUID     
 |        |
 +--------+--------+--------+------...
          |
          |
 ...------+
```

## License

[BSD-3-Clause](https://opensource.org/licenses/BSD-3-Clause)
