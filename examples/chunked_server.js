const http = require('http');
const { URL } = require('url');

const PORT = Number(process.env.PORT || 3001);
const CHUNK_DELAY_MS = Number(process.env.CHUNK_DELAY_MS || 120);

// Deliberately uneven chunk sizes: a client that decodes Transfer-Encoding
// correctly must not assume chunks line up with its own read buffer.
const CHUNKS = [
  'Menemen chunked transfer demo\n',
  '--\n',
  'Chunk sizes here are intentionally uneven, ',
  'so a decoder that assumes one chunk per read will drop bytes.\n',
  'x'.repeat(1024) + '\n',
  'tiny\n',
  'z'.repeat(4096) + '\n',
  'done\n'
];

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function sendJson(res, statusCode, payload) {
  res.writeHead(statusCode, {
    'Content-Type': 'application/json; charset=utf-8',
    'Cache-Control': 'no-store',
    'Access-Control-Allow-Origin': '*'
  });
  res.end(JSON.stringify(payload));
}

// Node uses chunked encoding automatically when no Content-Length is set and
// the body is written in pieces.
async function sendChunked(res, chunks, delayMs) {
  res.writeHead(200, {
    'Content-Type': 'text/plain; charset=utf-8',
    'Cache-Control': 'no-store',
    'Transfer-Encoding': 'chunked',
    'Access-Control-Allow-Origin': '*'
  });

  for (const chunk of chunks) {
    if (res.writableEnded || res.destroyed) {
      return;
    }
    res.write(chunk);
    if (delayMs > 0) {
      await sleep(delayMs);
    }
  }

  res.end();
}

const server = http.createServer(async (req, res) => {
  const parsed = new URL(req.url, `http://localhost:${PORT}`);

  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, OPTIONS'
    });
    res.end();
    return;
  }

  // Streams the chunks back-to-back with no artificial delay.
  if (parsed.pathname === '/chunked' && req.method === 'GET') {
    await sendChunked(res, CHUNKS, 0);
    return;
  }

  // Same body, but paced so the client actually has to wait between chunks.
  if (parsed.pathname === '/chunked-slow' && req.method === 'GET') {
    await sendChunked(res, CHUNKS, CHUNK_DELAY_MS);
    return;
  }

  // A single chunk, to cover the trivial case.
  if (parsed.pathname === '/chunked-one' && req.method === 'GET') {
    await sendChunked(res, ['only one chunk\n'], 0);
    return;
  }

  // Zero chunks: headers, then the terminating empty chunk and nothing else.
  if (parsed.pathname === '/chunked-empty' && req.method === 'GET') {
    await sendChunked(res, [], 0);
    return;
  }

  if (parsed.pathname === '/health') {
    sendJson(res, 200, { ok: true });
    return;
  }

  sendJson(res, 404, {
    error: 'Not found',
    routes: [
      'GET /chunked',
      'GET /chunked-slow',
      'GET /chunked-one',
      'GET /chunked-empty',
      'GET /health'
    ]
  });
});

server.listen(PORT, () => {
  const total = CHUNKS.reduce((sum, chunk) => sum + Buffer.byteLength(chunk), 0);
  console.log(`Chunked server listening on http://localhost:${PORT}`);
  console.log(`Chunked endpoint: GET /chunked  (${CHUNKS.length} chunks, ${total} bytes)`);
  console.log('Paced variant:    GET /chunked-slow');
});
