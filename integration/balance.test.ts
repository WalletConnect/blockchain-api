import { getTestSetup } from './init';

describe('Account balance', () => {
  const { baseUrl, projectId, httpClient } = getTestSetup();

  const fulfilled_address = '0xf3ea39310011333095CFCcCc7c4Ad74034CABA63'
  const empty_address = '0x5b6262592954B925B510651462b63ddEbcc22eaD'
  const currency = 'usd'
  const sdk_version = '4.1.9'

  it('fulfilled balance address', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_address}/balance?projectId=${projectId}&currency=${currency}`,
      {
        headers: {
            'x-sdk-version': sdk_version,
        }
      }
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.balances).toBe('object')
    expect(resp.data.balances.length).toBeGreaterThan(1)

    for (const item of resp.data.balances) {
      expect(typeof item.name).toBe('string')
      expect(typeof item.symbol).toBe('string')
      expect(item.chainId).toEqual(expect.stringMatching(/^(eip155:)?\d+$/))
      if (item.address !== undefined) {
        expect(item.address).toEqual(expect.stringMatching(/^(eip155:\d+:0x[0-9a-fA-F]{40})$/))
      } else {
        expect(item.address).toBeUndefined()
      }
      expect(typeof item.price).toBe('number')
      expect(typeof item.quantity).toBe('object')
      expect(typeof item.iconUrl).toBe('string')
    }
  })

  it('fulfilled balance address no version header', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${fulfilled_address}/balance?projectId=${projectId}&currency=${currency}`
    )
    // We should expect the empty balance response for the sdk version prior to 4.1.9
    // that doesn't send the x-sdk-version header due to the bug in the SDK
    expect(resp.status).toBe(200)
    expect(typeof resp.data.balances).toBe('object')
    expect(resp.data.balances).toHaveLength(0)
  })

  it('empty balance address', async () => {
    let resp: any = await httpClient.get(
      `${baseUrl}/v1/account/${empty_address}/balance?projectId=${projectId}&currency=${currency}`,
      {
        headers: {
            'x-sdk-version': sdk_version,
        }
      }
    )
    expect(resp.status).toBe(200)
    expect(typeof resp.data.balances).toBe('object')
    expect(resp.data.balances).toHaveLength(0)
  })

  it('force update balance for the ERC20 token', async () => {
    // USDC token contract address on Base
    const token_contract_address = 'eip155:8453:0x833589fcd6edb6e08f4c7c32d4f71b54bda02913'
    const endpoint = `/v1/account/${fulfilled_address}/balance`;
    const queryParams = `?projectId=${projectId}&currency=${currency}&forceUpdate=${token_contract_address}`;
    const url = `${baseUrl}${endpoint}${queryParams}`;
    const headers = {
        'x-sdk-version': sdk_version,
    };
    let resp = await httpClient.get(url, { headers });
    expect(resp.status).toBe(200)
    expect(typeof resp.data.balances).toBe('object')
    expect(resp.data.balances.length).toBeGreaterThan(1)

    for (const item of resp.data.balances) {
      expect(typeof item.name).toBe('string')
      expect(typeof item.symbol).toBe('string')
      expect(item.chainId).toEqual(expect.stringMatching(/^(eip155:)?\d+$/))
      if (item.address !== undefined) {
        expect(item.address).toEqual(expect.stringMatching(/^(eip155:\d+:0x[0-9a-fA-F]{40})$/))
      } else {
        expect(item.address).toBeUndefined()
      }
      expect(typeof item.price).toBe('number')
      expect(typeof item.quantity).toBe('object')
      expect(typeof item.iconUrl).toBe('string')
    }
  })

  it('force update balance for the native token', async () => {
    // ETH token
    // We are using `0xe...` as a contract address for native tokens
    const token_contract_address = 'eip155:1:0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
    const endpoint = `/v1/account/${fulfilled_address}/balance`;
    const queryParams = `?projectId=${projectId}&currency=${currency}&forceUpdate=${token_contract_address}`;
    const url = `${baseUrl}${endpoint}${queryParams}`;
    const headers = {
        'x-sdk-version': sdk_version,
    };
    let resp = await httpClient.get(url, { headers });
    expect(resp.status).toBe(200)
    expect(typeof resp.data.balances).toBe('object')
    expect(resp.data.balances.length).toBeGreaterThan(1)

    for (const item of resp.data.balances) {
      expect(typeof item.name).toBe('string')
      expect(typeof item.symbol).toBe('string')
      expect(item.chainId).toEqual(expect.stringMatching(/^(eip155:)?\d+$/))
      if (item.address !== undefined) {
        expect(item.address).toEqual(expect.stringMatching(/^(eip155:\d+:0x[0-9a-fA-F]{40})$/))
      } else {
        expect(item.address).toBeUndefined()
      }
      expect(typeof item.price).toBe('number')
      expect(typeof item.quantity).toBe('object')
      expect(typeof item.iconUrl).toBe('string')
    }
  })
})
