import { useEffect, useState } from "react";

type Metrics = {
  cpu_usage_percent: number;
  memory_used_mb: number;
  memory_total_mb: number;
};

function App() {
  const [metrics, setMetrics] = useState<Metrics | null>(null);

  useEffect(() => {
    fetch("http://localhost:3001/metrics")
      .then((res) => res.json())
      .then((data) => setMetrics(data))
      .catch((err) => console.error(err));
  }, []);

  if (!metrics) {
    return <h1>Loading...</h1>;
  }

  return (
    <div style={{ padding: "40px", fontFamily: "Arial" }}>
      <h1>⚡ System Monitor</h1>

      <h2>CPU Usage: {metrics.cpu_usage_percent}%</h2>

      <h2>
        Memory: {metrics.memory_used_mb} MB / {metrics.memory_total_mb} MB
      </h2>
    </div>
  );
}

export default App;