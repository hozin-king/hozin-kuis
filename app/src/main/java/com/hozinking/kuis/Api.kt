package com.hozinking.kuis

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.text.Html
import org.json.JSONObject
import java.io.BufferedReader
import java.io.IOException
import java.io.InputStreamReader
import java.net.HttpURLConnection
import java.net.URL

data class TriviaQuestion(val text: String, val correct: String, val choices: List<String>)

data class PokemonInfo(
    val name: String,
    val types: List<String>,
    val heightDm: Int,
    val weightHg: Int,
    val spriteUrl: String?
)

data class PokemonRound(
    val info: PokemonInfo,
    val choices: List<String>,
    val sprite: Bitmap?
)

/** Jaringan ke API gratis tanpa kunci: Open Trivia DB + PokeAPI. */
object Api {

    private fun get(urlStr: String): String {
        val conn = URL(urlStr).openConnection() as HttpURLConnection
        conn.connectTimeout = 15000
        conn.readTimeout = 15000
        conn.setRequestProperty("User-Agent", "HozinKuis/1.0")
        try {
            if (conn.responseCode != 200) throw IOException("HTTP ${conn.responseCode}")
            val br = BufferedReader(InputStreamReader(conn.inputStream, Charsets.UTF_8))
            return br.use { it.readText() }
        } finally {
            conn.disconnect()
        }
    }

    private fun decode(s: String): String =
        Html.fromHtml(s, Html.FROM_HTML_MODE_LEGACY).toString()

    fun fetchTrivia(category: Int, difficulty: String): List<TriviaQuestion> {
        val json = JSONObject(
            get("https://opentdb.com/api.php?amount=10&category=$category&difficulty=$difficulty&type=multiple")
        )
        if (json.optInt("response_code", -1) != 0) throw IOException("OpenTDB response_code != 0")
        val arr = json.getJSONArray("results")
        val out = ArrayList<TriviaQuestion>()
        for (i in 0 until arr.length()) {
            val o = arr.getJSONObject(i)
            val correct = decode(o.getString("correct_answer"))
            val choices = ArrayList<String>()
            choices.add(correct)
            val inc = o.getJSONArray("incorrect_answers")
            for (j in 0 until inc.length()) choices.add(decode(inc.getString(j)))
            choices.shuffle()
            out.add(TriviaQuestion(decode(o.getString("question")), correct, choices))
        }
        return out
    }

    fun fetchPokemonNames(): List<String> {
        val json = JSONObject(get("https://pokeapi.co/api/v2/pokemon?limit=1010"))
        val arr = json.getJSONArray("results")
        val out = ArrayList<String>()
        for (i in 0 until arr.length()) out.add(arr.getJSONObject(i).getString("name"))
        return out
    }

    fun fetchPokemon(id: Int): PokemonInfo {
        val json = JSONObject(get("https://pokeapi.co/api/v2/pokemon/$id"))
        val types = ArrayList<String>()
        val tarr = json.getJSONArray("types")
        for (i in 0 until tarr.length()) {
            types.add(tarr.getJSONObject(i).getJSONObject("type").getString("name"))
        }
        val sprite = json.optJSONObject("sprites")
            ?.optString("front_default", "").orEmpty()
            .takeIf { it.isNotEmpty() }
        return PokemonInfo(
            name = json.getString("name"),
            types = types,
            heightDm = json.getInt("height"),
            weightHg = json.getInt("weight"),
            spriteUrl = sprite
        )
    }

    fun downloadBitmap(urlStr: String): Bitmap? {
        val conn = URL(urlStr).openConnection() as HttpURLConnection
        conn.connectTimeout = 15000
        conn.readTimeout = 15000
        conn.setRequestProperty("User-Agent", "HozinKuis/1.0")
        try {
            if (conn.responseCode != 200) return null
            return BitmapFactory.decodeStream(conn.inputStream)
        } finally {
            conn.disconnect()
        }
    }
}
