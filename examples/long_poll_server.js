const http = require('http');
const { URL } = require('url');

const PORT = Number(process.env.PORT || 3000);
const POLL_TIMEOUT_MS = Number(process.env.POLL_TIMEOUT_MS || 25000);

const channels = new Map();
const waitersByChannel = new Map();

function getEvents(channel) {
  if (!channels.has(channel)) {
    channels.set(channel, []);
  }
  return channels.get(channel);
}

function getWaiters(channel) {
  if (!waitersByChannel.has(channel)) {
    waitersByChannel.set(channel, new Set());
  }
  return waitersByChannel.get(channel);
}

function sendJson(res, statusCode, payload) {
  res.writeHead(statusCode, {
    'Content-Type': 'application/json; charset=utf-8',
    'Cache-Control': 'no-store',
    'Access-Control-Allow-Origin': '*'
  });
  res.end(JSON.stringify(payload));
}

function parseBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (chunk) => chunks.push(chunk));
    req.on('end', () => {
      if (chunks.length === 0) {
        resolve({});
        return;
      }

      const raw = Buffer.concat(chunks).toString('utf8');
      try {
        resolve(JSON.parse(raw));
      } catch (error) {
        reject(new Error('Invalid JSON body'));
      }
    });
    req.on('error', reject);
  });
}

function nextId(channel) {
  return getEvents(channel).length;
}

function publish(channel, message) {
  const events = getEvents(channel);
  const event = {
    id: nextId(channel),
    message,
    timestamp: new Date().toISOString()
  };

  events.push(event);

  const waiters = getWaiters(channel);
  for (const waiter of waiters) {
    if (event.id >= waiter.cursor) {
      clearTimeout(waiter.timeout);
      sendJson(waiter.res, 200, {
        channel,
        events: events.slice(waiter.cursor),
        nextCursor: events.length,
        timeout: false
      });
      waiters.delete(waiter);
    }
  }

  return event;
}

function longPoll(req, res, url) {
  const channel = url.searchParams.get('channel') || 'chat';
  const rawCursor = url.searchParams.get('cursor') || '0';
  const cursor = Number(rawCursor);

  if (!Number.isInteger(cursor) || cursor < 0) {
    sendJson(res, 400, { error: 'cursor must be a non-negative integer' });
    return;
  }

  const events = getEvents(channel);

  if (events.length > cursor) {
    sendJson(res, 200, {
      channel,
      events: events.slice(cursor),
      nextCursor: events.length,
      timeout: false
    });
    return;
  }

  const waiters = getWaiters(channel);
  const waiter = {
    cursor,
    res,
    timeout: setTimeout(() => {
      sendJson(res, 200, {
        channel,
        events: [],
        nextCursor: cursor,
        timeout: true
      });
      waiters.delete(waiter);
    }, POLL_TIMEOUT_MS)
  };

  waiters.add(waiter);

  req.on('close', () => {
    clearTimeout(waiter.timeout);
    waiters.delete(waiter);
  });
}

async function handlePublish(req, res, url) {
  const method = req.method || 'GET';

  try {
    let channel = url.searchParams.get('channel') || 'chat';
    let message = url.searchParams.get('message');

    if (method === 'POST') {
      const body = await parseBody(req);
      channel = body.channel || channel;
      message = body.message || message;
    }

    if (!message) {
      sendJson(res, 400, { error: 'message is required' });
      return;
    }

    const event = publish(channel, String(message));
    sendJson(res, 200, {
      ok: true,
      event,
      channel
    });
  } catch (error) {
    sendJson(res, 400, { error: error.message || 'publish failed' });
  }
}

const server = http.createServer(async (req, res) => {
  const parsed = new URL(req.url || '/', `http://${req.headers.host}`);

  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET,POST,OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type'
    });
    res.end();
    return;
  }

  if (parsed.pathname === '/long-poll' && req.method === 'GET') {
    longPoll(req, res, parsed);
    return;
  }

  if (parsed.pathname === '/publish' && (req.method === 'GET' || req.method === 'POST')) {
    await handlePublish(req, res, parsed);
    return;
  }

  if (parsed.pathname === '/health') {
    sendJson(res, 200, { ok: true });
    return;
  }

  sendJson(res, 404, {
    error: 'Not found',
    routes: [
      'GET /long-poll?channel=chat&cursor=0',
      'GET /publish?channel=chat&message=hello',
      'POST /publish {"channel":"chat","message":"hello"}',
      'GET /health'
    ]
  });
});

server.listen(PORT, () => {
  console.log(`Long-poll server listening on http://localhost:${PORT}`);
  console.log('Poll endpoint:  GET /long-poll?channel=chat&cursor=0');
  console.log('Publish endpoint: GET /publish?channel=chat&message=hello');
});
