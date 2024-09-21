# af_xdp_test
af_xdp with libbpf-rs example

> [xsk-rs](https://github.com/DouglasGray/xsk-rs/tree/v0.6.1)
# run step
```shell
cd libxdp-sys
git clone https://github.com/xdp-project/xdp-tools.git
cd xdp-tools
git submodule update --init --recursive
git submodule foreach 'git fetch && git checkout master && git pull'

```
Change ifindex in the main.rs and then execute "cargo run"
