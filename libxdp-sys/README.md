# xdp-tools-sys
A Rust wrapper for libxdp that is compatible with [libbpf-rs](https://github.com/libbpf/libbpf-rs).

# error: libbpf: bpf_object_open_opts has non-zero extra bytes
The libbpf version in libxdp is outdated. You need to run the following commands:
```shell
cd xdp-tools
git submodule update --init --recursive
git submodule foreach 'git fetch && git checkout master && git pull'
```
