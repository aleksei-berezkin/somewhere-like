'use client'

import { useEffect, useReducer, useState } from 'react';
import { CitySearchResponseItem, fetchApi } from './api';

const resultsId = 'search-results'

export function SearchCity() {
  const [query, setQuery] = useState('')

  return <div className='search-city'>
    <form>
      <input
        name='query'
        type='text'
        placeholder='City'
        value={query}
        onChange={e => setQuery(e.target.value)}
        aria-controls={resultsId}
      />
    </form>
    <SearchResultList query={query} />
  </div>
}

type State = 
  | { name: 'delay' } & WithQuery
  | { name: 'fetch' } & WithQuery
  | { name: 'done', results: CitySearchResponseItem[] } & WithQuery
  | { name: 'failed' } & WithQuery

type Action =
  | { name: 'changedQuery', } & WithQuery
  | { name: 'delayFinished' } & WithQuery
  | { name: 'fetchFinished', results: CitySearchResponseItem[] } & WithQuery
  | { name: 'fetchFailed' } & WithQuery

type WithQuery = {
  query: string
}

function reducer(state: State, action: Action): State {
  if (action.name === 'changedQuery') {
    const newQuery = action.query.trim()
    if (newQuery === state.query) return state

    return newQuery
      ? { name: 'delay', query: newQuery }
      : { name: 'done', query: '', results: [] }
  }

  if (action.name === 'delayFinished' && state.name === 'delay' && action.query === state.query) {
    return { name: 'fetch', query: state.query }
  }

  if (action.name === 'fetchFinished' && state.name === 'fetch' && action.query === state.query) {
    return { name: 'done', query: state.query, results: action.results }
  }

  if (action.name === 'fetchFailed' && state.name === 'fetch' && action.query === state.query) {
    return { name: 'failed', query: state.query }
  }

  return state
}

function SearchResultList(p: { query: string }) {
  const [state, dispatch] = useReducer(reducer, { name: 'done', query: '', results: [] })
  const [displayedResults, setDisplayedResults] = useState<CitySearchResponseItem[]>([])

  useEffect(() => {
    dispatch({ name: 'changedQuery', query: p.query })
  }, [p.query])

  useEffect(() => {
    if (state.name === 'delay') {
      setTimeout(
        () =>
          dispatch({ name: 'delayFinished', query: state.query }),
        300,
      )
    } else if (state.name === 'fetch') {
      fetchApi({
        command: 'searchCity',
        query: state.query,
        maxItems: 10,
      }).then(response => {
        dispatch({ name: 'fetchFinished', query: state.query, results: response.items })
      }).catch(() => {
        dispatch({ name: 'fetchFailed', query: state.query })
      })
    } else if (state.name === 'done') {
      setDisplayedResults(state.results)
    }
  }, [state])

  return <>
    <nav>
      {
        displayedResults.map(item =>
          <p key={item.id}>{item.name}, {item.adminUnit ?? ''}, {item.country}</p>
        )
      }
    </nav>
    {
      state.name === 'failed' && <p>Server unavailable</p>
    }
  </>
}
