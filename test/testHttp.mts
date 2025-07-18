import assert from 'node:assert';
import { spawn } from 'node:child_process';
import { test } from 'uvu';
import { fetchApi } from './api.mts';

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
        maxItems: 3
    })

    assert(response.cacheHitRatePercent > 10)
    assert.equal(response.items.length, 3)

    const [firstItem] = response.items
    assert.equal(firstItem.matchedName, 'Tokyo')
    assert.equal(firstItem.name, 'Tokyo')
    assert.equal(firstItem.population, 9733276)
    assert.equal(firstItem.country, 'Japan')
})

test.after(() => void subprocess.kill())
test.run()
