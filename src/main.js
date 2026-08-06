const express = require("express");
const axios = require("axios");

const app = express();
app.use(express.json());

const backends = [
  "https://web-update-alert.onrender.com/api/internships",
  "https://web-update-alert.onrender.com/api/internships",
  "https://web-update-alert.onrender.com",
];
let current = 0;

function nextBackend() {
  const backend = backends[current];
  current = (current + 1) % backends.length;
  return backend;
}

app.get("/api/internships", async (req, res) => {
  const backend = nextBackend();

  try {
    const response = await axios.get(
      backend + "/api/internships",
      {
        params: req.query,
        validateStatus: () => true,
      }
    );

    res.status(response.status).json(response.data);
  } catch (err) {
    console.error(err.message);
    res.status(502).json({
      error: "Backend unavailable",
      backend,
    });
  }
});

const PORT = process.env.PORT || 8080;

app.listen(PORT, "0.0.0.0", () => {
  console.log(`Load Balancer running on port ${PORT}`);
});
