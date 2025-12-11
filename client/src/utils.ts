import ccxt from "ccxt";

/**
 * Generate signed Binance API request URLs for multiple accounts.
 *
 * Each account must send **two** requests in strict order:
 * 1. `GET /papi/v1/um/positionRisk`
 * 2. `GET /papi/v1/balance`
 *
 * ⚠️ **Important:** For every account, always send the `positionRisk` request first,
 * immediately followed by the `balance` request.
 *
 * ## Endpoint Notes
 *
 * - **GET /papi/v1/um/positionRisk**
 *   - Do **not** include the `symbol` parameter.
 *   - Use a larger `recvWindow`, e.g. `60000` (milliseconds).
 *
 * - **GET /papi/v1/balance**
 *   - Do **not** include the `asset` parameter.
 *   - Use a larger `recvWindow`, e.g. `60000` (milliseconds).
 *
 * ## Example Usage
 *
 * ```js
 * const origRequests = [
 *   {
 *     url: "https://papi.binance.com/papi/v1/um/positionRisk?recvWindow=60000&timestamp=1760921486287&signature=f792e...",
 *     headers: {
 *       'X-MBX-APIKEY': 'nzI7iU3YRLlO1olZG8xsTQnnRQPcxKfrY...'
 *     }
 *   },
 *   {
 *     url: "https://papi.binance.com/papi/v1/balance?recvWindow=60000&timestamp=1760921486287&signature=f362...",
 *     headers: {
 *       'X-MBX-APIKEY': 'nzI7iU3YRLlO1olZG8xsTQnnRQPcxKfrY...'
 *     }
 *   }
 * ];
 *
 * const { requests, responseResolves } = makeBinanceRequestParams(origRequests);
 * ```
 */
export function makeBinanceRequestParams(origRequests: any[]) {
  const RISK_URL = "https://papi.binance.com/papi/v1/um/positionRisk";
  const BALANCE_URL = "https://papi.binance.com/papi/v1/balance";

  if (!Array.isArray(origRequests) || origRequests.length < 2 || origRequests.length % 2 !== 0) {
    throw new Error("❌ Invalid input: 'origRequests' must be an even-length array (pairs of requests per account).");
  }

  const requests = [];
  const responseResolves = [];

  for (let i = 0; i < origRequests.length; i++) {
    const origRequest = origRequests[i];
    const isRisk = i % 2 === 0;
    const expectedUrl = isRisk ? RISK_URL : BALANCE_URL;

    if (!origRequest?.url?.startsWith(expectedUrl)) {
      throw new Error(`❌ Invalid order at index ${i}: expected positionRisk request first, then balance request.`);
    }

    requests.push({
      url: origRequest.url,
      method: "GET",
      header: { ...origRequest.headers },
      body: "",
    });

    responseResolves.push([
      {
        keyName: `${i}`,
        parseType: "json",
        parsePath: "$",
        op: "SHA256_EX",
      },
    ]);
  }

  return { requests, responseResolves };
}

export function makeBinanceSpotRequestParams(origRequests: any[]) {
  if (!Array.isArray(origRequests) || origRequests.length < 1) {
    throw new Error("❌ Invalid input: 'origRequests' must be greater than 1.");
  }

  const requests = [];
  const responseResolves = [];

  for (let i = 0; i < origRequests.length; i++) {
    const origRequest = origRequests[i];
    requests.push({
      url: origRequest.url,
      method: "GET",
      header: { ...origRequest.headers },
      body: "",
    });

    responseResolves.push([
      {
        keyName: `${i}`,
        parseType: "json",
        parsePath: "$",
        op: "SHA256_EX",
      },
    ]);
  }

  return { requests, responseResolves };
}

export function getBinanceAccounts() {
  const accounts = [];

  const key = process.env.BINANCE_API_KEY;
  const secret = process.env.BINANCE_API_SECRET;
  if (key && secret) {
    accounts.push({ key, secret });
  }
  for (let i = 1; i <= 100; i++) {
    const key = process.env[`BINANCE_API_KEY${i}`];
    const secret = process.env[`BINANCE_API_SECRET${i}`];
    if (key && secret) {
      accounts.push({ key, secret });
    }
  }

  if (accounts.length === 0) {
    throw new Error("Please configure at least one set of BINANCE_API_KEY{i} / BINANCE_API_SECRET{i} in .env.");
  }

  const seen = new Set();
  for (const acc of accounts) {
    if (seen.has(`${acc.key}${acc.secret}`)) {
      throw new Error(`Duplicate BINANCE_API_KEY{i} detected`);
    }
    seen.add(`${acc.key}${acc.secret}`);
  }
  return accounts;
}

export function makeBinanaceOrigRequests(accounts: any[]) {
  const recvWindow = Number(process.env.BINANCE_RECV_WINDOW) || 60;
  let signParams = { recvWindow: recvWindow * 1000 };

  let origRequests = []
  for (const acc of accounts) {
    const exchange = new ccxt['binance']({
      apiKey: acc.key,
      secret: acc.secret,
    });

    let umPositionRiskRequest = exchange.sign('um/positionRisk', 'papi', 'GET', signParams);
    let balanceRequest = exchange.sign('balance', 'papi', 'GET', signParams);
    origRequests.push(umPositionRiskRequest);
    origRequests.push(balanceRequest);
  }

  return origRequests;
}

export function makeBinanaceSpotOrigRequests(accounts: any[]) {
  const recvWindow = Number(process.env.BINANCE_RECV_WINDOW) || 60;
  let signParams = { recvWindow: recvWindow * 1000 };

  let origRequests = []
  for (const acc of accounts) {
    const exchange = new ccxt['binance']({
      apiKey: acc.key,
      secret: acc.secret,
    });

    let spotBalanceRequest = exchange.sign('account', 'private', 'GET', { ...signParams, omitZeroBalances: true });
    origRequests.push(spotBalanceRequest);
  }

  return origRequests;
}
