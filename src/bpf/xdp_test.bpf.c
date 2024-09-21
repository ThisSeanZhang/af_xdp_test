
#include "vmlinux.h"

#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>
#include <bpf/bpf_core_read.h>

char LICENSE[] SEC("license") = "Dual BSD/GPL";

struct
{
    __uint(type, BPF_MAP_TYPE_XSKMAP);
    __type(key, __u32);
    __type(value, __u32);
    __uint(max_entries, 64);
} xsks_map SEC(".maps");

SEC("xdp")
int xdp_pass(struct xdp_md *ctx)
{
    int index = ctx->rx_queue_index;
    // bpf_printk("ingress_ifindex: %d", ctx->ingress_ifindex);
    // bpf_printk("egress_ifindex: %d", ctx->egress_ifindex);
    int *xsk_fd = bpf_map_lookup_elem(&xsks_map, &index);

    if (xsk_fd)
    {
        int result = bpf_redirect_map(&xsks_map, index, XDP_ABORTED);
        return result;
    }

    return XDP_PASS;
}