export async function fetchApi<Req extends CityRequest>(request: Req): Promise<GetResponseType<Req>> {
    return Promise.race([
        fetchApiImpl(request),
        new Promise<GetResponseType<Req>>((_, reject) => setTimeout(() => reject(new Error('Timeout')), 5000)),
    ])
}

async function fetchApiImpl<Req extends CityRequest>(request: Req): Promise<GetResponseType<Req>> {
    const req = new Request('http://localhost:3001', {
        method: 'POST',
        body: JSON.stringify(request),
    })
    const res = await fetch(req);
    return await res.json();
}

type GetResponseType<R extends CityRequest> =
    R extends CitySearchRequest ? CitySearchResponse
    : R extends ClimateSearchRequest ? ClimateSearchResponse
    : never

export type CityRequest =
    | CitySearchRequest
    | ClimateSearchRequest

export type CitySearchRequest = {
    command: 'searchCity'
    query: string
    startIndex?: number
    maxItems?: number
}

export type ClimateSearchRequest = {
    command: 'searchClimate'
    cityId: number
    startIndex?: number
    maxItems?: number
}

export type CityResponse =
    | CitySearchResponse
    | ClimateSearchResponse

export type CitySearchResponse = {
    command: 'searchCity'
    items: CitySearchResponseItem[]
    elapsedMs: number
    cacheHitRatePercent: number
}

export type CitySearchResponseItem = {
    id: number
    score: number
    matchedName: string
    name: string
    population: number
    adminUnit: string | null
    country: string
}

export type ClimateSearchResponse = {
    command: 'searchClimate'
    items: ClimateSearchResponseItem[]
    elapsedMs: number
}

export type ClimateSearchResponseItem = {
    id: number
    city: City
    distanceKm: number
    similarityPercent: number
}

export type City = {
    names: string[]
    latitude: number
    longitude: number
    adminUnit: string | null
    country: string
    population: number
    elevation: number | null
    region: string
    modificationDate: string
    climate: CityClimate
}

export type CityClimate = {
    humidityMonthly: number[]
    pptMonthly: Monthly
    sradMonthly: Monthly
    tmaxMonthly: Monthly
    tminMonthly: Monthly
    wsMonthly: Monthly
}

export type Monthly = [
    jan: number,
    feb: number,
    mar: number,
    apr: number,
    may: number,
    jun: number,
    jul: number,
    aug: number,
    sep: number,
    oct: number,
    nov: number,
    dec: number,
]
