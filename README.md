# fmu-rs

![workflow](https://github.com/avrabe/fmu-rs/actions/workflows/rust.yml/badge.svg)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs?ref=badge_shield)
[![codecov](https://codecov.io/gh/avrabe/fmu-rs/branch/main/graph/badge.svg?token=bqz07qp5a3)](https://codecov.io/gh/avrabe/fmu-rs)

fmu-rs is a Rust implementation of FullMetalUpdate which handles update for the applications on a system on which it is running.

## Building the application

On Ubuntu 20.04 install rust and the needed external dependencies (libostree)

```bash
sudo apt-get install build-essential libostree-dev 
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Afterwards just build with the regular cargo commands

```bash
cd fmu-rs
cargo check
cargo build
cargo run
```

## Setting up the server

For example install an ubuntu 20.04 Server with following additional applications.

```bash
sudo apt-get install docker.io socat 
sudo useradd -a -G docker $USER
curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube-linux-amd64 /usr/local/bin/minikube
```

Start the kubernetes engine and ensure it is configured properly.

```bash
minikube start --driver=docker
minikube addons enable ingress
```

Then to enforce all the files to setup the needed servers.

```bash
cd fmu-rs
minikube kubectl -- apply -f backend
sudo socat TCP-LISTEN:80,fork TCP:$(minikube ip):80 &
sudo socat TCP-LISTEN:30022,fork TCP:$(minikube ip):30022 &
```

And to configure some example id's.

```bash
./tools/ConfigServer.sh
```

## Access

From the outside
| Service / Container | URL | Login |
|---|---|---|
| hawkBit Update Server | [http://localhost:80/](http://localhost:80/) |  admin:admin |
| Ostree Server - http| [http://localhost:80/ostree](http://localhost:80/ostree) | - |
| Ostree Server - ssh| ssh localhost -p 30022 | root:root |

Only within the kubernetes cluster
| Service / Container | URL | Login |
|---|---|---|
| MySQL | localhost:3306/hawkbit | root |
| RabbitMQ | [http://localhost:15672](http://localhost:15672) | guest:guest |

## License

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs?ref=badge_large)
