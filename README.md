# capy

working repository for a distributed programming library modelled around object capability protocols like OCapN.

## components

- `kadc` is a modified implementation of the Kademlia distributed hash table protocol (originally implemented [here](http://github.com/vwxi/kad))
- `kadcd` is a daemon that allows applications to access the network indirectly
- `capy` is a library that will allow for RPC between peers that speak `kadc` through `kadcd`