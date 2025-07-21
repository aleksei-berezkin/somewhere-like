import assert from 'node:assert'
import { spawn } from 'node:child_process'
import { test } from 'uvu'
import { fetchApi } from './api.mts'

const subprocess = spawn('cargo', ['run', '-p', 'backend', '--release', '--bin', 'http'])

test.before(() => new Promise(resolve => {
    subprocess.stderr.on('data', data => {
        if (data.includes('Listening on')) {
            resolve(undefined)
        }
    })
}))

test('simple search', async () => {
    const response = await fetchApi({
        command: 'searchCity',
        query: 'Tokyo',
        maxItems: 3,
    })

    assert(response.cacheHitRatePercent > 10)
    assert.equal(response.items.length, 3)

    const [firstItem] = response.items
    assert.equal(firstItem.matchedName, 'Tokyo')
    assert.equal(firstItem.name, 'Tokyo')
    assert.equal(firstItem.population, 9733276)
    assert.equal(firstItem.country, 'Japan')
})

test('search with admin unit', async () => {
    const response = await fetchApi({
        command: 'searchCity',
        query: 'paris texas',
    })

    assert(response.cacheHitRatePercent > 50)

    const [firstItem] = response.items
    assert.equal(firstItem.matchedName, 'Paris')
    assert.equal(firstItem.name, 'Paris')
    assert.equal(firstItem.adminUnit, 'Texas')
    assert.equal(firstItem.country, 'United States')
})

test('search with country', async () => {
    const response = await fetchApi({
        command: 'searchCity',
        query: 'berlin salvador',
    })

    assert(response.cacheHitRatePercent > 50)

    const [firstItem] = response.items
    assert.equal(firstItem.matchedName, 'Berlin')
    assert.equal(firstItem.name, 'Berlín')
    assert.equal(firstItem.adminUnit, 'Usulután')
    assert.equal(firstItem.country, 'El Salvador')
})

test.after(() => void subprocess.kill())
test.run()
