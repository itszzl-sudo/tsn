# TypeScript \+ Native IoT CLI Demo 设计方案

# TypeScript \+ Native IoT CLI Demo 设计方案

我给你设计一个**开箱即用、纯 TS 编写、可直接运行**的 IoT 场景 CLI 工具，基于 **Node\.js \+ TypeScript**，模拟物联网设备管理场景（设备上报、指令下发、状态查询、日志监控）。

## 场景定位

模拟**工业 / 智能家居 IoT 网关 CLI 工具**，功能：

- 扫描局域网 IoT 设备

- 查看设备状态（温湿度、开关、电量）

- 下发控制指令（开灯、关灯、重启）

- 模拟设备数据上报

- 设备日志实时查看

- 配置设备连接参数

## 技术栈

- 语言：**TypeScript**

- 运行环境：Node\.js

- CLI 框架：`commander`（业界标准）

- 交互：`inquirer`（交互式选择）

- 模拟 IoT 设备：内存数据库（无需真实硬件）

- 构建：tsc \+ npm scripts

---

# 完整 Demo 代码（可直接复制运行）

## 1\. 项目结构

```Plain Text
iot-cli/
├── src/
│   ├── cli.ts           # CLI 入口
│   ├── device/          # 设备模拟模块
│   │   └── mockDevice.ts
│   ├── commands/        # 命令注册
│   │   ├── scan.ts
│   │   ├── status.ts
│   │   ├── control.ts
│   │   └── log.ts
├── package.json
└── tsconfig.json
```

## 2\. package\.json

```json
{
  "name": "ts-iot-cli",
  "version": "1.0.0",
  "description": "TypeScript IoT 设备管理 CLI Demo",
  "main": "dist/cli.js",
  "bin": {
    "iot": "./dist/cli.js"
  },
  "scripts": {
    "build": "tsc",
    "start": "node dist/cli.js",
    "dev": "ts-node src/cli.ts"
  },
  "dependencies": {
    "commander": "^12.0.0",
    "inquirer": "^9.2.15"
  },
  "devDependencies": {
    "@types/node": "^20.11.0",
    "ts-node": "^10.9.2",
    "typescript": "^5.3.3"
  }
}
```

## 3\. tsconfig\.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "CommonJS",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true
  },
  "include": ["src/**/*"]
}
```

## 4\. src/device/mockDevice\.ts（模拟 IoT 设备）

```typescript
// 模拟 IoT 设备数据结构
export interface IoTDevice {
  id: string;
  name: string;
  type: 'light' | 'sensor' | 'gateway';
  online: boolean;
  temperature?: number;
  humidity?: number;
  power: number;
  switchStatus?: boolean;
  lastReport: string;
}

// 模拟设备列表
export const MOCK_DEVICES: IoTDevice[] = [
  {
    id: 'dev_001',
    name: '客厅灯',
    type: 'light',
    online: true,
    power: 85,
    switchStatus: false,
    lastReport: new Date().toISOString(),
  },
  {
    id: 'dev_002',
    name: '卧室温湿度传感器',
    type: 'sensor',
    online: true,
    temperature: 24.5,
    humidity: 48,
    power: 92,
    lastReport: new Date().toISOString(),
  },
  {
    id: 'dev_003',
    name: '网关主机',
    type: 'gateway',
    online: true,
    power: 100,
    lastReport: new Date().toISOString(),
  },
];

// 获取所有设备
export function getDevices() {
  return MOCK_DEVICES;
}

// 根据 ID 查询设备
export function getDeviceById(id: string) {
  return MOCK_DEVICES.find(d => d.id === id);
}

// 下发控制指令
export function controlDevice(id: string, action: 'on' | 'off' | 'restart') {
  const device = getDeviceById(id);
  if (!device) return null;

  if (device.type === 'light') {
    if (action === 'on') device.switchStatus = true;
    if (action === 'off') device.switchStatus = false;
  }

  if (action === 'restart') {
    device.online = false;
    setTimeout(() => {
      device.online = true;
      device.lastReport = new Date().toISOString();
    }, 1500);
  }

  device.lastReport = new Date().toISOString();
  return device;
}
```

## 5\. src/commands/scan\.ts（扫描设备）

```typescript
import { getDevices } from '../device/mockDevice';

export function scanCommand() {
  console.log('\n🔍 正在扫描局域网 IoT 设备...\n');
  
  setTimeout(() => {
    const devices = getDevices();
    devices.forEach(dev => {
      console.log(`✅ [${dev.id}] ${dev.name} | 类型：${dev.type} | 状态：${dev.online ? '在线' : '离线'}`);
    });
    console.log(`\n扫描完成，共发现 ${devices.length} 台设备`);
  }, 800);
}
```

## 6\. src/commands/status\.ts（查询状态）

```typescript
import { getDeviceById } from '../device/mockDevice';

export function statusCommand(deviceId: string) {
  const device = getDeviceById(deviceId);

  if (!device) {
    console.log(`❌ 设备 ${deviceId} 不存在`);
    return;
  }

  console.log('\n📊 设备状态详情：');
  console.log(`ID：${device.id}`);
  console.log(`名称：${device.name}`);
  console.log(`类型：${device.type}`);
  console.log(`在线：${device.online ? '是' : '否'}`);
  if (device.temperature) console.log(`温度：${device.temperature}℃`);
  if (device.humidity) console.log(`湿度：${device.humidity}%`);
  if (device.switchStatus !== undefined) console.log(`开关：${device.switchStatus ? '开启' : '关闭'}`);
  console.log(`电量：${device.power}%`);
  console.log(`最后上报：${new Date(device.lastReport).toLocaleString()}\n`);
}
```

## 7\. src/commands/control\.ts（指令下发）

```typescript
import { controlDevice } from '../device/mockDevice';

export async function controlCommand(deviceId: string, action: string) {
  const validActions = ['on', 'off', 'restart'];
  
  if (!validActions.includes(action)) {
    console.log('❌ 支持的指令：on | off | restart');
    return;
  }

  const result = controlDevice(deviceId, action as any);
  if (!result) {
    console.log(`❌ 设备 ${deviceId} 不存在`);
    return;
  }

  console.log(`\� 指令已下发：${action}`);
  console.log(`✅ 设备 ${deviceId} 执行成功\n`);
}
```

## 8\. src/commands/log\.ts（实时日志）

```typescript
export function logCommand(deviceId?: string) {
  console.log(`\n📝 实时日志监控（设备：${deviceId || '全部'}）`);
  console.log('---------------------------------------');

  let count = 0;
  const timer = setInterval(() => {
    const time = new Date().toLocaleTimeString();
    console.log(`[${time}] ${deviceId || 'all'} → 数据正常上报`);
    count++;
    if (count > 5) {
      clearInterval(timer);
      console.log('\n🛑 日志监控结束\n');
    }
  }, 1000);
}
```

## 9\. src/cli\.ts（CLI 主入口）

```typescript
#!/usr/bin/env node
import { Command } from 'commander';
import { scanCommand } from './commands/scan';
import { statusCommand } from './commands/status';
import { controlCommand } from './commands/control';
import { logCommand } from './commands/log';

const program = new Command();

program
  .name('iot')
  .description('TS IoT 设备管理 CLI 工具')
  .version('1.0.0');

// 扫描设备
program.command('scan')
  .description('扫描局域网 IoT 设备')
  .action(scanCommand);

// 查询状态
program.command('status <deviceId>')
  .description('查询设备状态')
  .action(statusCommand);

// 控制指令
program.command('control <deviceId> <action>')
  .description('控制设备 on｜off｜restart')
  .action(controlCommand);

// 查看日志
program.command('log [deviceId]')
  .description('查看设备实时日志')
  .action(logCommand);

program.parse();
```

---

# 运行步骤

## 1\. 安装依赖

```bash
npm install
```

## 2\. 编译 TS

```bash
npm run build
```

## 3\. 链接到全局命令

```bash
npm link
```

## 4\. 使用 CLI

```bash
# 扫描设备
iot scan

# 查看设备状态
iot status dev_001

# 下发指令
iot control dev_001 on
iot control dev_001 off
iot control dev_001 restart

# 查看日志
iot log
iot log dev_002
```

---

# 功能演示效果

```Plain Text
$ iot scan

🔍 正在扫描局域网 IoT 设备...

✅ [dev_001] 客厅灯 | 类型：light | 状态：在线
✅ [dev_002] 卧室温湿度传感器 | 类型：sensor | 状态：在线
✅ [dev_003] 网关主机 | 类型：gateway | 状态：在线

扫描完成，共发现 3 台设备
```

```Plain Text
$ iot status dev_002

📊 设备状态详情：
ID：dev_002
名称：卧室温湿度传感器
类型：sensor
在线：是
温度：24.5℃
湿度：48%
电量：92%
最后上报：2025/4/1 22:10:15
```

---

# 可扩展方向（真实 IoT 场景可用）

- MQTT 接入（真实物联网协议）

- HTTP/CoAP 设备接口调用

- 设备配置文件（json/yml）

- 连接真实硬件（串口、BLE、Modbus）

- 数据上报云端（阿里云 / 腾讯云 IoT）

- 设备批量管理、定时任务、告警

---

# 总结

这个 Demo 是**标准 TypeScript CLI \+ IoT 场景**的最佳实践模板：

- 纯 TS 编写，结构清晰

- 模块化设计，易于扩展

- 模拟真实设备交互逻辑

- 可直接对接真实硬件 / 云平台

- 开箱即用，无需额外配置

> （注：文档部分内容可能由 AI 生成）
