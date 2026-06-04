/**
 * TS-Native Metric API Server
 * 原生HTTP服务器，提供指标查询API
 */

import { http_listen, http_stop, route_get, route_post, response_json, response_text } from "tsnp-metric-api";

// 指标数据存储
const metrics: Record<string, number> = {
    cpu_usage: 45.2,
    memory_usage: 67.8,
    disk_usage: 52.3,
    network_in: 12345,
    network_out: 6789,
    request_count: 523
};

// GET /metrics - 返回所有指标
route_get("/metrics", "handle_metrics");

function handle_metrics(): void {
    const json = JSON.stringify({
        timestamp: Date.now(),
        metrics: metrics
    });
    response_json(json);
}

// GET /metrics/:name - 返回单个指标
route_get("/metrics/:name", "handle_single_metric");

function handle_single_metric(name: string): void {
    const value = metrics[name];
    if (value === undefined) {
        response_json(JSON.stringify({ error: "Metric not found" }));
    } else {
        response_json(JSON.stringify({
            name: name,
            value: value,
            timestamp: Date.now()
        }));
    }
}

// POST /metrics - 更新指标
route_post("/metrics", "handle_update_metrics");

function handle_update_metrics(body: string): void {
    try {
        const data = JSON.parse(body);
        for (const key in data) {
            if (metrics[key] !== undefined) {
                metrics[key] = data[key];
            }
        }
        response_json(JSON.stringify({ success: true, message: "Metrics updated" }));
    } catch (e) {
        response_json(JSON.stringify({ error: "Invalid JSON" }));
    }
}

// GET /health - 健康检查
route_get("/health", "handle_health");

function handle_health(): void {
    response_json(JSON.stringify({
        status: "ok",
        uptime: Date.now(),
        version: "1.0.0"
    }));
}

// 启动服务器
function main(): void {
    const PORT = 8080;
    console.log("========================================");
    console.log("  TS-Native Metric API Server");
    console.log("  原生二进制 · 高性能 · 零依赖");
    console.log("========================================");
    console.log("");
    console.log(`服务器启动在端口: ${PORT}`);
    console.log("");
    console.log("可用端点:");
    console.log("  GET  /metrics          - 获取所有指标");
    console.log("  GET  /metrics/:name    - 获取单个指标");
    console.log("  POST /metrics          - 更新指标");
    console.log("  GET  /health           - 健康检查");
    console.log("");
    console.log("按 Ctrl+C 停止服务器");
    console.log("");
    
    http_listen(PORT);
}

main();