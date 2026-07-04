use axum::{routing::get, Router};
use sysinfo::{System, Disks, Networks};
use std::net::SocketAddr;
use axum::response::Html;

async fn metrics() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.global_cpu_info().cpu_usage();
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let free_mem = sys.free_memory();

    let cores: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();

    let mut processes: Vec<(String, u64, f32)> = sys.processes()
        .values()
        .map(|p| (p.name().to_string(), p.memory() / 1024 / 1024, p.cpu_usage()))
        .collect();
    processes.sort_by(|a, b| b.1.cmp(&a.1));
    processes.truncate(10);

    let disks = Disks::new_with_refreshed_list();
    let disk_info: Vec<serde_json::Value> = disks.iter().map(|d| {
        serde_json::json!({
            "name": d.name().to_string_lossy(),
            "total_gb": d.total_space() / 1024 / 1024 / 1024,
            "free_gb": d.available_space() / 1024 / 1024 / 1024,
            "used_gb": (d.total_space() - d.available_space()) / 1024 / 1024 / 1024,
        })
    }).collect();

    let networks = Networks::new_with_refreshed_list();
    let net_info: Vec<serde_json::Value> = networks.iter().take(4).map(|(name, data)| {
        serde_json::json!({
            "name": name,
            "rx_mb": data.total_received() / 1024 / 1024,
            "tx_mb": data.total_transmitted() / 1024 / 1024,
        })
    }).collect();

    let uptime = System::uptime();
    let hours = uptime / 3600;
    let minutes = (uptime % 3600) / 60;

    serde_json::json!({
        "cpu_usage_percent": format!("{:.1}", cpu),
        "memory_used_mb": used_mem / 1024 / 1024,
        "memory_total_mb": total_mem / 1024 / 1024,
        "memory_free_mb": free_mem / 1024 / 1024,
        "cpu_cores": cores,
        "top_processes": processes,
        "disks": disk_info,
        "networks": net_info,
        "uptime": format!("{}h {}m", hours, minutes),
        "os": System::name().unwrap_or_default(),
        "kernel": System::kernel_version().unwrap_or_default(),
        "hostname": System::host_name().unwrap_or_default(),
    }).to_string()
}

async fn dashboard() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>System Monitor</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/4.4.1/chart.umd.min.js"></script>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        :root {
            --sidebar-w: 220px;
            --titlebar-h: 52px;
            --bg: #1e1e1e;
            --sidebar: #2a2a2a;
            --surface: #2d2d2d;
            --surface2: #333333;
            --border: #3a3a3a;
            --text: #e8e8e8;
            --muted: #888;
            --accent: #0a84ff;
            --green: #30d158;
            --orange: #ff9f0a;
            --red: #ff453a;
            --purple: #bf5af2;
        }

        
body.light {
    --bg: #f5f5f7;
    --sidebar: #ffffff;
    --surface: #ffffff;
    --surface2: #f5f5f7;
    --border: #d2d2d7;
    --text: #1d1d1f;
    --muted: #86868b;
    --accent: #0071e3;
    --green: #1db954;
    --orange: #f56300;
    --red: #d70015;
    --purple: #9b59b6;
}


        html, body { height: 100%; overflow: hidden; }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', sans-serif;
            background: var(--bg);
            color: var(--text);
            display: flex;
            flex-direction: column;
            font-size: 13px;
        }

        /* TITLEBAR */
        .titlebar {
            height: var(--titlebar-h);
            background: var(--sidebar);
            display: flex;
            align-items: center;
            padding: 0 1rem;
            gap: 1rem;
            border-bottom: 1px solid var(--border);
            -webkit-app-region: drag;
            flex-shrink: 0;
        }
        .traffic-lights { display: flex; gap: 7px; }
        .tl {
            width: 13px; height: 13px; border-radius: 50%;
            cursor: pointer; flex-shrink: 0;
        }
        .tl-red { background: #ff5f56; border: 1px solid #e0443e; }
        .tl-yellow { background: #ffbd2e; border: 1px solid #dea123; }
        .tl-green { background: #27c93f; border: 1px solid #1aab29; }
        .titlebar-title {
            flex: 1;
            text-align: center;
            font-size: 13px;
            font-weight: 600;
            color: var(--text);
            margin-right: 60px;
        }
        .live-pill {
            background: rgba(48,209,88,0.15);
            color: var(--green);
            padding: 2px 10px;
            border-radius: 99px;
            font-size: 11px;
            font-weight: 600;
            display: flex;
            align-items: center;
            gap: 5px;
        }
        .dot { width: 5px; height: 5px; background: var(--green); border-radius: 50%; animation: pulse 2s infinite; }
        @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.2} }

        /* LAYOUT */
        .app-body { display: flex; flex: 1; overflow: hidden; }

        /* SIDEBAR */
        .sidebar {
            width: var(--sidebar-w);
            background: var(--sidebar);
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            padding: 1rem 0;
            flex-shrink: 0;
            overflow-y: auto;
        }
        .sidebar-section-label {
            font-size: 10px;
            font-weight: 700;
            color: var(--muted);
            text-transform: uppercase;
            letter-spacing: 0.1em;
            padding: 0.6rem 1rem 0.3rem;
        }
        .sidebar-item {
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 6px 1rem;
            cursor: pointer;
            border-radius: 8px;
            margin: 1px 6px;
            color: var(--muted);
            font-size: 13px;
            transition: all 0.1s;
        }
        .sidebar-item:hover { background: var(--surface2); color: var(--text); }
        .sidebar-item.active { background: var(--accent); color: #fff; }
        .sidebar-item .icon { font-size: 14px; width: 18px; text-align: center; }
        .sidebar-bottom {
            margin-top: auto;
            padding: 1rem;
            border-top: 1px solid var(--border);
        }
        .sys-info-item { margin-bottom: 6px; }
        .sys-info-label { font-size: 10px; color: var(--muted); }
        .sys-info-val { font-size: 12px; color: var(--text); font-weight: 500; }

        /* MAIN CONTENT */
        .content {
            flex: 1;
            overflow-y: auto;
            padding: 1.5rem;
            display: none;
        }
        .content.active { display: block; }

        /* SECTION TITLE */
        .section-title {
            font-size: 20px;
            font-weight: 700;
            color: var(--text);
            margin-bottom: 1.2rem;
            letter-spacing: -0.02em;
        }

        /* CARDS */
        .grid-4 { display: grid; grid-template-columns: repeat(4,1fr); gap: 0.8rem; margin-bottom: 1rem; }
        .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 0.8rem; margin-bottom: 1rem; }
        .grid-3 { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 0.8rem; margin-bottom: 1rem; }

        .card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 1.1rem;
        }
        .card-label { font-size: 10px; font-weight: 700; color: var(--muted); text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: 0.4rem; }
        .card-value { font-size: 2rem; font-weight: 700; letter-spacing: -0.04em; line-height: 1; margin-bottom: 0.8rem; }
        .bar-track { background: var(--bg); border-radius: 99px; height: 4px; overflow: hidden; margin-bottom: 0.3rem; }
        .bar-fill { height: 100%; border-radius: 99px; transition: width 0.6s ease; }
        .bar-sub { display: flex; justify-content: space-between; font-size: 10px; color: var(--muted); }

        /* CHART CARDS */
        .chart-card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 1.1rem;
        }
        .chart-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.8rem; }
        .chart-title { font-size: 11px; font-weight: 700; color: var(--muted); text-transform: uppercase; letter-spacing: 0.1em; }
        .chart-val { font-size: 13px; font-weight: 600; color: var(--text); }

        /* TABLE */
        .mac-table { width: 100%; border-collapse: collapse; }
        .mac-table th {
            text-align: left;
            font-size: 10px;
            font-weight: 700;
            color: var(--muted);
            text-transform: uppercase;
            letter-spacing: 0.08em;
            padding: 0 0.5rem 0.6rem;
            border-bottom: 1px solid var(--border);
        }
        .mac-table td {
            padding: 6px 0.5rem;
            font-size: 12px;
            border-bottom: 1px solid rgba(255,255,255,0.04);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            max-width: 180px;
        }
        .mac-table tr:last-child td { border-bottom: none; }

        /* CORE GRID */
        .cores-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(65px,1fr)); gap: 6px; }
        .core-cell {
            background: var(--surface2);
            border-radius: 8px;
            padding: 8px 6px;
            text-align: center;
        }
        .core-label { font-size: 9px; color: var(--muted); margin-bottom: 3px; }
        .core-val { font-size: 14px; font-weight: 700; }

        /* DISK + NET ROWS */
        .row-item { padding: 8px 0; border-bottom: 1px solid var(--border); }
        .row-item:last-child { border-bottom: none; }
        .row-header { display: flex; justify-content: space-between; margin-bottom: 5px; font-size: 12px; }
        .row-name { font-weight: 500; color: var(--text); }
        .row-sub { color: var(--muted); font-size: 11px; }

        .net-row { display: flex; justify-content: space-between; align-items: center; padding: 7px 0; border-bottom: 1px solid var(--border); font-size: 12px; }
        .net-row:last-child { border-bottom: none; }
        .net-badges { display: flex; gap: 8px; }
        .badge { padding: 2px 8px; border-radius: 6px; font-size: 11px; font-weight: 600; }
        .badge-green { background: rgba(48,209,88,0.15); color: var(--green); }
        .badge-blue { background: rgba(10,132,255,0.15); color: var(--accent); }

        
#theme-btn {
    background: var(--surface2);
    color: var(--text);
    border:1px solid var(--border);
    border-radius:8px;
    padding:6px 12px;
    cursor:pointer;
    margin-right:10px;
}

/* SCROLLBAR */
        ::-webkit-scrollbar { width: 6px; }
        ::-webkit-scrollbar-track { background: transparent; }
        ::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }
    </style>
</head>
<body>

<!-- TITLEBAR -->
<div class="titlebar">
    <div class="traffic-lights">
        <div class="tl tl-red"></div>
        <div class="tl tl-yellow"></div>
        <div class="tl tl-green"></div>
    </div>
    <div class="titlebar-title">System Monitor</div>
    <button id="theme-btn" onclick="toggleTheme()">☀️ Light</button>
    <div class="live-pill"><div class="dot"></div>Live</div>
    <span style="font-size:11px;color:var(--muted);margin-left:8px" id="last-updated">--</span>
</div>

<div class="app-body">

    <!-- SIDEBAR -->
    <div class="sidebar">
        <div class="sidebar-section-label">Monitor</div>
        <div class="sidebar-item active" onclick="switchTab('overview')">
            <span class="icon">📊</span> Overview
        </div>
        <div class="sidebar-item" onclick="switchTab('cpu')">
            <span class="icon">🖥️</span> CPU
        </div>
        <div class="sidebar-item" onclick="switchTab('memory')">
            <span class="icon">🧠</span> Memory
        </div>
        <div class="sidebar-item" onclick="switchTab('processes')">
            <span class="icon">⚙️</span> Processes
        </div>
        <div class="sidebar-section-label">System</div>
        <div class="sidebar-item" onclick="switchTab('disk')">
            <span class="icon">💾</span> Disk
        </div>
        <div class="sidebar-item" onclick="switchTab('network')">
            <span class="icon">🌐</span> Network
        </div>
        <div class="sidebar-section-label">API</div>
        <div class="sidebar-item" onclick="window.open('/metrics','_blank')">
            <span class="icon">🔗</span> JSON API
        </div>

        <div class="sidebar-bottom">
            <div class="sys-info-item"><div class="sys-info-label">Hostname</div><div class="sys-info-val" id="s-hostname">--</div></div>
            <div class="sys-info-item"><div class="sys-info-label">OS</div><div class="sys-info-val" id="s-os">--</div></div>
            <div class="sys-info-item"><div class="sys-info-label">Uptime</div><div class="sys-info-val" id="s-uptime">--</div></div>
            <div class="sys-info-item"><div class="sys-info-label">Kernel</div><div class="sys-info-val" id="s-kernel">--</div></div>
        </div>
    </div>

    <!-- OVERVIEW TAB -->
    <div class="content active" id="tab-overview">
        <div class="section-title">Overview</div>
        <div class="grid-4">
            <div class="card">
                <div class="card-label">CPU</div>
                <div class="card-value" id="o-cpu" style="color:var(--green)">--</div>
                <div class="bar-track"><div class="bar-fill" id="o-cpu-bar" style="background:var(--green);width:0%"></div></div>
                <div class="bar-sub"><span>Usage</span><span>100%</span></div>
            </div>
            <div class="card">
                <div class="card-label">Memory</div>
                <div class="card-value" id="o-mem" style="color:var(--accent)">--</div>
                <div class="bar-track"><div class="bar-fill" id="o-mem-bar" style="background:var(--accent);width:0%"></div></div>
                <div class="bar-sub"><span id="o-mem-used">--</span><span id="o-mem-total">--</span></div>
            </div>
            <div class="card">
                <div class="card-label">Free RAM</div>
                <div class="card-value" id="o-free" style="color:var(--orange)">--</div>
                <div class="bar-track"><div class="bar-fill" id="o-free-bar" style="background:var(--orange);width:0%"></div></div>
                <div class="bar-sub"><span>Available</span><span id="o-free-pct">--</span></div>
            </div>
            <div class="card">
                <div class="card-label">Cores</div>
                <div class="card-value" id="o-cores" style="color:var(--purple)">--</div>
                <div class="bar-track"><div class="bar-fill" style="background:var(--purple);width:100%"></div></div>
                <div class="bar-sub"><span>CPU cores</span><span>active</span></div>
            </div>
        </div>
        <div class="grid-2">
            <div class="chart-card">
                <div class="chart-header"><span class="chart-title">CPU History</span><span class="chart-val" id="o-cpu-now">--</span></div>
                <canvas id="o-cpu-chart" height="90"></canvas>
            </div>
            <div class="chart-card">
                <div class="chart-header"><span class="chart-title">Memory History</span><span class="chart-val" id="o-mem-now">--</span></div>
                <canvas id="o-mem-chart" height="90"></canvas>
            </div>
        </div>
        <div class="grid-2">
            <div class="chart-card">
                <div class="chart-header"><span class="chart-title">Top Processes</span></div>
                <table class="mac-table">
                    <thead><tr><th>Process</th><th>CPU</th><th>Memory</th></tr></thead>
                    <tbody id="o-processes"></tbody>
                </table>
            </div>
            <div class="chart-card">
                <div class="chart-header"><span class="chart-title">Network</span></div>
                <div id="o-network"></div>
            </div>
        </div>
    </div>

    <!-- CPU TAB -->
    <div class="content" id="tab-cpu">
        <div class="section-title">CPU</div>
        <div class="grid-2" style="margin-bottom:1rem">
            <div class="chart-card">
                <div class="chart-header"><span class="chart-title">CPU Usage History</span><span class="chart-val" id="c-cpu-val">--</span></div>
                <canvas id="c-cpu-chart" height="120"></canvas>
            </div>
            <div class="card" style="display:flex;flex-direction:column;justify-content:center;gap:1rem">
                <div><div class="card-label">Current Usage</div><div class="card-value" id="c-cpu" style="color:var(--green)">--</div></div>
                <div class="bar-track" style="height:6px"><div class="bar-fill" id="c-cpu-bar" style="background:var(--green);width:0%"></div></div>
                <div><div class="card-label">Core Count</div><div style="font-size:1.4rem;font-weight:700;color:var(--purple)" id="c-cores">--</div></div>
            </div>
        </div>
        <div class="chart-card">
            <div class="chart-header"><span class="chart-title">Per-Core Usage</span></div>
            <div class="cores-grid" id="c-cores-grid"></div>
        </div>
    </div>

    <!-- MEMORY TAB -->
    <div class="content" id="tab-memory">
        <div class="section-title">Memory</div>
        <div class="grid-3" style="margin-bottom:1rem">
            <div class="card"><div class="card-label">Used</div><div class="card-value" id="m-used" style="color:var(--accent)">--</div><div class="bar-track"><div class="bar-fill" id="m-used-bar" style="background:var(--accent);width:0%"></div></div><div class="bar-sub"><span id="m-used-mb">--</span><span>used</span></div></div>
            <div class="card"><div class="card-label">Free</div><div class="card-value" id="m-free" style="color:var(--green)">--</div><div class="bar-track"><div class="bar-fill" id="m-free-bar" style="background:var(--green);width:0%"></div></div><div class="bar-sub"><span id="m-free-mb">--</span><span>free</span></div></div>
            <div class="card"><div class="card-label">Total</div><div class="card-value" style="color:var(--muted)" id="m-total">--</div><div class="bar-track"><div class="bar-fill" style="background:var(--muted);width:100%"></div></div><div class="bar-sub"><span>installed RAM</span><span></span></div></div>
        </div>
        <div class="chart-card">
            <div class="chart-header"><span class="chart-title">Memory Usage History</span><span class="chart-val" id="m-mem-now">--</span></div>
            <canvas id="m-mem-chart" height="130"></canvas>
        </div>
    </div>

    <!-- PROCESSES TAB -->
    <div class="content" id="tab-processes">
        <div class="section-title">Processes</div>
        <div class="chart-card">
            <table class="mac-table">
                <thead><tr><th style="width:200px">Process</th><th>CPU %</th><th>Memory</th></tr></thead>
                <tbody id="p-processes"></tbody>
            </table>
        </div>
    </div>

    <!-- DISK TAB -->
    <div class="content" id="tab-disk">
        <div class="section-title">Disk</div>
        <div class="chart-card">
            <div id="disk-list"></div>
        </div>
    </div>

    <!-- NETWORK TAB -->
    <div class="content" id="tab-network">
        <div class="section-title">Network</div>
        <div class="chart-card">
            <div id="net-list"></div>
        </div>
    </div>

</div>

<script>
    const N = 30;
    const labels = Array(N).fill('');
    const cpuH = Array(N).fill(0);
    const memH = Array(N).fill(0);

    function makeChart(id, color, height=90) {
        return new Chart(document.getElementById(id), {
            type: 'line',
            data: { labels, datasets: [{ data: [...(color === '#30d158' ? cpuH : memH)], borderColor: color, backgroundColor: color.replace(')', ',0.1)').replace('rgb','rgba'), borderWidth: 1.5, fill: true, tension: 0.4 }] },
            options: {
                responsive: true,
                plugins: { legend: { display: false } },
                scales: {
                    x: { display: false },
                    y: { min: 0, max: 100, grid: { color: 'rgba(255,255,255,0.04)' }, ticks: { color: '#888', font: { size: 10 }, callback: v => v + '%' } }
                },
                elements: { point: { radius: 0 } },
                animation: { duration: 300 }
            }
        });
    }

    const charts = {
        oCpu: makeChart('o-cpu-chart', '#30d158'),
        oMem: makeChart('o-mem-chart', '#0a84ff'),
        cCpu: makeChart('c-cpu-chart', '#30d158'),
        mMem: makeChart('m-mem-chart', '#0a84ff'),
    };

    function colorFor(p) {
        if (p < 50) return '#30d158';
        if (p < 80) return '#ff9f0a';
        return '#ff453a';
    }

    function updateCharts(cpu, memPct) {
        cpuH.push(cpu); cpuH.shift();
        memH.push(memPct); memH.shift();
        Object.values(charts).forEach(c => {
            c.data.datasets[0].data = c.canvas.id.includes('mem') ? [...memH] : [...cpuH];
            c.update();
        });
    }

    function switchTab(name) {
        document.querySelectorAll('.content').forEach(el => el.classList.remove('active'));
        document.querySelectorAll('.sidebar-item').forEach(el => el.classList.remove('active'));
        document.getElementById('tab-' + name).classList.add('active');
        event.currentTarget.classList.add('active');
    }

    async function fetchStats() {
        try {
            const res = await fetch('/metrics');
            const d = await res.json();

            document.getElementById('last-updated').textContent = new Date().toLocaleTimeString();
            document.getElementById('s-hostname').textContent = d.hostname || '--';
            document.getElementById('s-os').textContent = d.os || '--';
            document.getElementById('s-uptime').textContent = d.uptime || '--';
            document.getElementById('s-kernel').textContent = (d.kernel || '--').substring(0, 15);

            const cpu = parseFloat(d.cpu_usage_percent);
            const usedMb = d.memory_used_mb;
            const totalMb = d.memory_total_mb;
            const freeMb = d.memory_free_mb;
            const memPct = (usedMb / totalMb * 100);
            const freePct = (freeMb / totalMb * 100);
            const cores = d.cpu_cores || [];
            const cpuColor = colorFor(cpu);

            // OVERVIEW
            document.getElementById('o-cpu').textContent = cpu.toFixed(1) + '%';
            document.getElementById('o-cpu').style.color = cpuColor;
            document.getElementById('o-cpu-bar').style.width = cpu + '%';
            document.getElementById('o-cpu-bar').style.background = cpuColor;
            document.getElementById('o-cpu-now').textContent = cpu.toFixed(1) + '%';
            document.getElementById('o-mem').textContent = memPct.toFixed(1) + '%';
            document.getElementById('o-mem-bar').style.width = memPct + '%';
            document.getElementById('o-mem-used').textContent = usedMb.toLocaleString() + ' MB';
            document.getElementById('o-mem-total').textContent = totalMb.toLocaleString() + ' MB';
            document.getElementById('o-mem-now').textContent = memPct.toFixed(1) + '%';
            document.getElementById('o-free').textContent = freeMb.toLocaleString();
            document.getElementById('o-free-bar').style.width = freePct + '%';
            document.getElementById('o-free-pct').textContent = freePct.toFixed(1) + '%';
            document.getElementById('o-cores').textContent = cores.length;

            // CPU TAB
            document.getElementById('c-cpu').textContent = cpu.toFixed(1) + '%';
            document.getElementById('c-cpu').style.color = cpuColor;
            document.getElementById('c-cpu-bar').style.width = cpu + '%';
            document.getElementById('c-cpu-bar').style.background = cpuColor;
            document.getElementById('c-cpu-val').textContent = cpu.toFixed(1) + '%';
            document.getElementById('c-cores').textContent = cores.length + ' cores';
            document.getElementById('c-cores-grid').innerHTML = cores.map((c, i) =>
                `<div class="core-cell"><div class="core-label">Core ${i+1}</div><div class="core-val" style="color:${colorFor(c)}">${parseFloat(c).toFixed(0)}%</div></div>`
            ).join('');

            // MEMORY TAB
            document.getElementById('m-used').textContent = memPct.toFixed(1) + '%';
            document.getElementById('m-used-bar').style.width = memPct + '%';
            document.getElementById('m-used-mb').textContent = usedMb.toLocaleString() + ' MB';
            document.getElementById('m-free').textContent = freePct.toFixed(1) + '%';
            document.getElementById('m-free-bar').style.width = freePct + '%';
            document.getElementById('m-free-mb').textContent = freeMb.toLocaleString() + ' MB';
            document.getElementById('m-total').textContent = (totalMb / 1024).toFixed(0) + ' GB';
            document.getElementById('m-mem-now').textContent = memPct.toFixed(1) + '%';

            // PROCESSES
            const procRows = (d.top_processes || []).map(p =>
                `<tr>
                    <td style="color:var(--text)">${p[0]}</td>
                    <td style="color:${colorFor(p[2])};font-weight:600">${parseFloat(p[2]).toFixed(1)}%</td>
                    <td style="color:var(--muted)">${p[1].toLocaleString()} MB</td>
                </tr>`
            ).join('');
            document.getElementById('o-processes').innerHTML = procRows;
            document.getElementById('p-processes').innerHTML = procRows;

            // NETWORK
            const netHtml = (d.networks || []).map(n =>
                `<div class="net-row">
                    <span style="color:var(--text);font-weight:500">${n.name}</span>
                    <div class="net-badges">
                        <span class="badge badge-green">↓ ${n.rx_mb} MB</span>
                        <span class="badge badge-blue">↑ ${n.tx_mb} MB</span>
                    </div>
                </div>`
            ).join('');
            document.getElementById('o-network').innerHTML = netHtml;
            document.getElementById('net-list').innerHTML = netHtml;

            // DISK
            const diskHtml = (d.disks || []).map(disk => {
                const pct = disk.total_gb > 0 ? (disk.used_gb / disk.total_gb * 100) : 0;
                return `<div class="row-item">
                    <div class="row-header">
                        <span class="row-name">${disk.name}</span>
                        <span class="row-sub">${disk.used_gb} GB used of ${disk.total_gb} GB</span>
                    </div>
                    <div class="bar-track" style="height:5px">
                        <div class="bar-fill" style="width:${pct}%;background:${colorFor(pct)}"></div>
                    </div>
                </div>`;
            }).join('');
            document.getElementById('disk-list').innerHTML = diskHtml;

            updateCharts(cpu, memPct);

        } catch(e) { console.error(e); }
    }

    
function toggleTheme() {
    const isLight=document.body.classList.toggle('light');
    document.getElementById('theme-btn').textContent=isLight?'🌙 Dark':'☀️ Light';
    localStorage.setItem('theme',isLight?'light':'dark');
}
document.addEventListener('DOMContentLoaded',()=>{
 if(localStorage.getItem('theme')==='light'){
   document.body.classList.add('light');
   const b=document.getElementById('theme-btn');
   if(b) b.textContent='🌙 Dark';
 }
});

fetchStats();
    setInterval(fetchStats, 3000);
</script>
</body>
</html>
    "#)
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap();

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/metrics", get(metrics));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on port {}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}