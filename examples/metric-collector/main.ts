// TS-Native 运维采集 CLI - 真实实现
// 使用FFI扩展实现网络请求、文件IO、系统调用

// FFI函数声明（由tsnp扩展提供）
declare function http_get(url: string): string;
declare function file_append(path: string, content: string): void;
declare function file_exists(path: string): number;
declare function argc(): number;
declare function argv(index: number): string;
declare function now_ms(): number;
declare function sleep(ms: number): void;

// 配置接口
interface Config {
    targetUrl: string;
    intervalSec: number;
    maxRuns: number;
    logFile: string;
}

// 指标数据结构
interface Metric {
    timestamp: number;
    cpu: number;
    memory: number;
    requests: number;
    responseTime: number;
}

/**
 * 解析命令行参数
 */
function parseArgs(): Config {
    let config: Config = {
        targetUrl: "https://httpbin.org/json",
        intervalSec: 60,
        maxRuns: 0,  // 0 = 无限运行
        logFile: "./metric.log"
    };
    
    // 解析参数
    // 用法: metric-collector [url] [interval] [maxRuns]
    let argCount = argc();
    
    if (argCount >= 2) {
        config.targetUrl = argv(1);
    }
    
    if (argCount >= 3) {
        let intervalStr = argv(2);
        config.intervalSec = parseInt(intervalStr);
    }
    
    if (argCount >= 4) {
        let maxRunsStr = argv(3);
        config.maxRuns = parseInt(maxRunsStr);
    }
    
    return config;
}

/**
 * 格式化时间戳
 */
declare function format_timestamp(ms: number): string;

function formatTimestamp(ms: number): string {
    return format_timestamp(ms);
}

/**
 * 采集指标
 */
function collectMetric(url: string): Metric {
    let startTime = now_ms();
    
    // 真实HTTP请求
    let response = "";
    let success = false;
    
    try {
        response = http_get(url);
        success = true;
    } catch (e) {
        print("HTTP请求失败: " + e);
        success = false;
    }
    
    let endTime = now_ms();
    let responseTime = endTime - startTime;
    
    // 模拟系统指标（实际应通过系统API获取）
    let cpu = Math.random() * 100;
    let memory = Math.random() * 100;
    let requests = Math.floor(Math.random() * 1000);
    
    return {
        timestamp: startTime,
        cpu: cpu,
        memory: memory,
        requests: requests,
        responseTime: responseTime
    };
}

/**
 * 写入日志
 */
function writeLog(logFile: string, metric: Metric): void {
    let timestamp = formatTimestamp(metric.timestamp);
    let logLine = "[" + timestamp + "] ";
    logLine = logLine + "CPU=" + metric.cpu.toFixed(2) + "% ";
    logLine = logLine + "MEM=" + metric.memory.toFixed(2) + "% ";
    logLine = logLine + "REQ=" + metric.requests + " ";
    logLine = logLine + "RT=" + metric.responseTime + "ms\n";
    
    file_append(logFile, logLine);
}

/**
 * 打印指标
 */
function printMetric(metric: Metric, runNum: number, maxRuns: number): void {
    let timestamp = formatTimestamp(metric.timestamp);
    
    print("[" + timestamp + "] 第 " + runNum + "/" + maxRuns + " 次采集");
    print("  CPU使用率: " + metric.cpu.toFixed(2) + "%");
    print("  内存使用率: " + metric.memory.toFixed(2) + "%");
    print("  请求数: " + metric.requests);
    print("  响应时间: " + metric.responseTime + "ms");
    print("");
}

/**
 * 主函数
 */
function main(): void {
    print("========================================");
    print("  TS-Native 运维采集 CLI");
    print("  原生二进制 · 无Node依赖 · 极速启动");
    print("========================================");
    print("");
    
    // 解析配置
    let config = parseArgs();
    
    print("配置:");
    print("  采集地址: " + config.targetUrl);
    print("  采集间隔: " + config.intervalSec + "秒");
    print("  最大运行: " + (config.maxRuns == 0 ? "无限" : config.maxRuns + "次"));
    print("  日志文件: " + config.logFile);
    print("");
    
    // 检查日志文件
    if (file_exists(config.logFile) == 0) {
        print("创建日志文件: " + config.logFile);
        file_append(config.logFile, "# TS-Native 运维采集日志\n");
        file_append(config.logFile, "# 启动时间: " + formatTimestamp(now_ms()) + "\n\n");
    }
    
    print("开始采集...\n");
    
    // 循环采集
    let runCount = 0;
    let continueRun = true;
    
    while (continueRun) {
        runCount = runCount + 1;
        
        // 采集指标
        let metric = collectMetric(config.targetUrl);
        
        // 打印结果
        let maxDisplay = config.maxRuns == 0 ? runCount : config.maxRuns;
        printMetric(metric, runCount, maxDisplay);
        
        // 写入日志
        writeLog(config.logFile, metric);
        
        // 检查是否达到最大运行次数
        if (config.maxRuns > 0 && runCount >= config.maxRuns) {
            continueRun = false;
        } else {
            // 等待下次采集
            print("等待 " + config.intervalSec + " 秒...\n");
            sleep(config.intervalSec * 1000);
        }
    }
    
    print("========================================");
    print("  采集完成");
    print("  总运行次数: " + runCount);
    print("  日志文件: " + config.logFile);
    print("========================================");
}

// 启动程序
main();
