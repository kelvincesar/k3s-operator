## Description

This is a test of interface with k3s creating a small operator which allows to read the pods and nodes, and migrate a pods from one node to another.
In this case we're deploying a small example from rancher, which is an HTTP server that allow us to see the pod id that handled the request.

## Commands

```sh
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
```

To see some logs:

```sh
sudo journalctl -u k3s -f
```

```sh
sudo journalctl -u k3s-agent -f
```


