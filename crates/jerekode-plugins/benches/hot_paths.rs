//! Criterion benchmark hooks for hot paths (see docs/perf-baseline.md).

use criterion::{Criterion, criterion_group, criterion_main};
use jerekode_plugins::{
    BunPluginHost, HookCall, InMemorySidecarPort, NativePluginHost, PluginOrchestrator,
    SidecarOutbound, SidecarPort,
};
use std::hint::black_box;
use std::sync::Arc;

fn bench_json_roundtrip(c: &mut Criterion) {
    c.bench_function("hook_call_json_roundtrip", |b| {
        let call = HookCall {
            hook: "before_transform".into(),
            payload: serde_json::json!({"input": "hello"}),
        };
        b.iter(|| {
            let json = serde_json::to_string(black_box(&call)).unwrap();
            let parsed: HookCall = serde_json::from_str(&json).unwrap();
            black_box(parsed);
        });
    });
}

fn bench_sidecar_in_memory(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("sidecar_in_memory_send_recv", |b| {
        b.to_async(&rt).iter(|| async {
            let port = InMemorySidecarPort::new();
            port.send(SidecarOutbound::Init {
                config: serde_json::json!({}),
                plugins: vec![],
            })
            .await
            .unwrap();
            let _ = port.recv().await.unwrap();
        });
    });
}

fn bench_orchestrator_dispatch(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("orchestrator_dispatch_hook", |b| {
        b.to_async(&rt).iter(|| async {
            let port = Arc::new(InMemorySidecarPort::new());
            let bun = Arc::new(BunPluginHost::new(port));
            let native = Arc::new(NativePluginHost::new());
            let orch = PluginOrchestrator::new(vec![native, bun]);
            let _ = orch
                .dispatch_hook(HookCall {
                    hook: "before_transform".into(),
                    payload: serde_json::json!({"input": "x"}),
                })
                .await
                .unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_json_roundtrip,
    bench_sidecar_in_memory,
    bench_orchestrator_dispatch
);
criterion_main!(benches);
