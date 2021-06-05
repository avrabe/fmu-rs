# fmu-rs
![workflow](https://github.com/avrabe/fmu-rs/actions/workflows/rust.yml/badge.svg)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs?ref=badge_shield)
[![codecov](https://codecov.io/gh/avrabe/fmu-rs/branch/main/graph/badge.svg?token=bqz07qp5a3)](https://codecov.io/gh/avrabe/fmu-rs)


fmu_rs is a Rust implementation of FullMetalUpdate which handles update for the System on which it is running.

It has been created in order to learn Rust by reimplementing an existing project used by the author.
The program can execute but not update anything yet.

```bash
sudo update-alternatives --set iptables /usr/sbin/iptables-legacy
sudo update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy
minikube start --driver=podman
minikube addons enable ingress
cd backend
minikube kubectl -- apply -f hawkbit-pod.yaml,hawkbit-service.yaml,hawknet-networkpolicy.yaml,mysql-pod.yaml,mysql-service.yaml,rabbitmq-pod.yaml,rabbitmq-service.yaml


```


```bash
sudo apt-get install docker.io socat 
sudo useradd -a -G docker $USER
curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube-linux-amd64 /usr/local/bin/minikube
```

```bash
minikube start --driver=docker
minikube addons enable ingress
```

```bash
cd fmu-rs
minikube kubectl -- apply -f backend
sudo socat TCP-LISTEN:80,fork TCP:$(minikube ip):80
```


Needs:
libostree-dev
## License
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Favrabe%2Ffmu-rs?ref=badge_large)