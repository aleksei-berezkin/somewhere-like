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

test('climate simple', async () => {
    const response = await fetchApi({
        command: 'searchClimate',
        cityId: 14823,
        maxItems: 2,
    })

    assert.equal(response.items.length, 2)

    const [firstItem] = response.items
    assert.equal(firstItem.id, 14823)
    assert.equal(firstItem.distanceKm, 0)
    assert.equal(firstItem.similarityPercent, 100)
    assert.equal(firstItem.city.names[0], 'Munich')
    assert(firstItem.city.names.includes('München'))
    assert.equal(firstItem.city.adminUnit, 'Bavaria')
    assert.equal(firstItem.city.country, 'Germany')
    assertMonthlyWithin(firstItem.city.climate.humidityMonthly, 50, 80)
    assertMonthlyWithin(firstItem.city.climate.pptMonthly, 40, 120)
    assertMonthlyWithin(firstItem.city.climate.sradMonthly, 30, 230)
    assertMonthlyWithin(firstItem.city.climate.tmaxMonthly, 4, 26)
    assertMonthlyWithin(firstItem.city.climate.tminMonthly, -3, 15)
    assertMonthlyWithin(firstItem.city.climate.wsMonthly, 2, 4)
})

function assertMonthlyWithin(actual: number[], min: number, max: number) {
    assert.equal(actual.length, 12)
    assert(
        actual.every(it => min <= it && it <= max),
        `actual: ${actual}, min: ${min}, max: ${max}`,
    )
}

test.after(() => void subprocess.kill())
test.run()
