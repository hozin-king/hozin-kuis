package com.hozinking.kuis

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.view.View
import android.widget.Button
import android.widget.TextView

class ResultActivity : Activity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_result)

        val mode = intent.getStringExtra("mode") ?: "trivia"
        val score = intent.getIntExtra("score", 0)
        val correct = intent.getIntExtra("correct", 0)
        val total = intent.getIntExtra("total", 10)
        val best = intent.getIntExtra("best", 0)
        val isNewBest = intent.getBooleanExtra("newBest", false)

        findViewById<TextView>(R.id.tvMode).text =
            if (mode == "trivia") "Trivia" else "Tebak Pokemon"
        findViewById<TextView>(R.id.tvScore).text = "Skor: $score"
        findViewById<TextView>(R.id.tvCorrect).text = "Benar: $correct/$total"
        findViewById<TextView>(R.id.tvBest).text = "Skor terbaik: $best"
        findViewById<TextView>(R.id.tvNewBest).visibility =
            if (isNewBest) View.VISIBLE else View.GONE

        findViewById<Button>(R.id.btnAgain).setOnClickListener {
            val intent = Intent(this, QuizActivity::class.java).apply {
                putExtra("mode", mode)
                putExtra("category", this@ResultActivity.intent.getIntExtra("category", 9))
                putExtra("difficulty", this@ResultActivity.intent.getStringExtra("difficulty") ?: "easy")
            }
            startActivity(intent)
            finish()
        }
        findViewById<Button>(R.id.btnMenu).setOnClickListener { finish() }
    }
}
