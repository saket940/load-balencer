const express = require("express");
const axios = require("axios");

const app = express();
app.use(express.json());

const backends = [
  "http://127.0.0.1:8001",
  "http://127.0.0.1:8002",
  "http://127.0.0.1:8003",
];

let current = 0;

function nextBackend() {
  const backend = backends[current];
  current = (current + 1) % backends.length;
  return backend;
}

app.all("*", async (req, res) => {
  const backend = nextBackend();

  try {
    const response = await axios({
      method: req.method,
      url: backend + req.originalUrl,
      data: req.body,
      headers: req.headers,
      validateStatus: () => true,
    });

    res.status(response.status).set(response.headers).send(response.data);
  } catch (err) {
    res.status(502).json({
      error: "Backend unavailable",
      backend,
    });
  }
});

app.listen(8080, () => {
  console.log("Load Balancer running on http://localhost:8080");
});
