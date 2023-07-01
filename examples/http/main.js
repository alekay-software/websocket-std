import express from "express"

const app = express()
const PORT = 3000


app.get("/test" , (req, res) => {
    const data = {
        "name": "Alejandro",
        "age": 24
    }
    
    res.status(200)
    res.json(data)
})

app.listen(PORT, () => {
    console.log(`Express listening on port ${PORT}`)
})