const request = new Request('http://localhost:3000', {
    method: "POST",
    body: JSON.stringify({
        command: 'searchCity',
        query: 'Tokyo',
        startIndex: 0,
        maxItems: 8
    }),
});

const res = await fetch(request);
const json = await res.json();
console.log(json);
