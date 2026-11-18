package com.hozinking.kuis

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.widget.ArrayAdapter
import android.widget.Button
import android.widget.Spinner
import android.widget.TextView

class MainActivity : Activity() {

    companion object {
        const val PREFS = "kuis_best"
        const val KEY_TRIVIA = "best_trivia"
        const val KEY_POKEMON = "best_pokemon"

        val CATEGORIES = listOf(
            "Umum" to 9,
            "Film" to 11,
            "Sains" to 17,
            "Komputer" to 18,
            "Olahraga" to 21,
            "Sejarah" to 23
        )
        val DIFFICULTIES = listOf(
            "Mudah" to "easy",
            "Sedang" to "medium",
            "Sulit" to "hard"
        )
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val spCategory = findViewById<Spinner>(R.id.spCategory)
        val spDifficulty = findViewById<Spinner>(R.id.spDifficulty)
        spCategory.adapter = ArrayAdapter(
            this, android.R.layout.simple_spinner_item,
            CATEGORIES.map { it.first }
        ).apply { setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item) }
        spDifficulty.adapter = ArrayAdapter(
            this, android.R.layout.simple_spinner_item,
            DIFFICULTIES.map { it.first }
        ).apply { setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item) }

        findViewById<Button>(R.id.btnTrivia).setOnClickListener {
            val intent = Intent(this, QuizActivity::class.java).apply {
                putExtra("mode", "trivia")
                putExtra("category", CATEGORIES[spCategory.selectedItemPosition].second)
                putExtra("difficulty", DIFFICULTIES[spDifficulty.selectedItemPosition].second)
            }
            startActivity(intent)
        }
        findViewById<Button>(R.id.btnPokemon).setOnClickListener {
            startActivity(Intent(this, QuizActivity::class.java).putExtra("mode", "pokemon"))
        }
    }

    override fun onResume() {
        super.onResume()
        val p = getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        findViewById<TextView>(R.id.tvBest).text =
            "Skor terbaik Trivia: ${p.getInt(KEY_TRIVIA, 0)}\n" +
            "Skor terbaik Pokemon: ${p.getInt(KEY_POKEMON, 0)}"
    }
}
