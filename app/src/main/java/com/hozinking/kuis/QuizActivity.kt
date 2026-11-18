package com.hozinking.kuis

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.os.Bundle
import android.os.CountDownTimer
import android.os.Handler
import android.os.Looper
import android.view.View
import android.widget.Button
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import kotlin.random.Random

class QuizActivity : Activity() {

    private val total = 10
    private var mode = "trivia"
    private var category = 9
    private var difficulty = "easy"

    private var trivia: List<TriviaQuestion> = emptyList()
    private var rounds: List<PokemonRound> = emptyList()
    private var index = 0
    private var score = 0
    private var correctCount = 0
    private var streak = 0
    private var answered = false

    private var timer: CountDownTimer? = null
    private val handler = Handler(Looper.getMainLooper())

    private lateinit var tvProgress: TextView
    private lateinit var tvScore: TextView
    private lateinit var tvStreak: TextView
    private lateinit var pbTimer: ProgressBar
    private lateinit var tvTimer: TextView
    private lateinit var loadingView: LinearLayout
    private lateinit var errorView: LinearLayout
    private lateinit var tvError: TextView
    private lateinit var quizView: LinearLayout
    private lateinit var ivSprite: ImageView
    private lateinit var tvHint: TextView
    private lateinit var tvQuestion: TextView
    private lateinit var answerBtns: List<Button>

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_quiz)

        mode = intent.getStringExtra("mode") ?: "trivia"
        category = intent.getIntExtra("category", 9)
        difficulty = intent.getStringExtra("difficulty") ?: "easy"

        tvProgress = findViewById(R.id.tvProgress)
        tvScore = findViewById(R.id.tvScore)
        tvStreak = findViewById(R.id.tvStreak)
        pbTimer = findViewById(R.id.pbTimer)
        tvTimer = findViewById(R.id.tvTimer)
        loadingView = findViewById(R.id.loadingView)
        errorView = findViewById(R.id.errorView)
        tvError = findViewById(R.id.tvError)
        quizView = findViewById(R.id.quizView)
        ivSprite = findViewById(R.id.ivSprite)
        tvHint = findViewById(R.id.tvHint)
        tvQuestion = findViewById(R.id.tvQuestion)
        answerBtns = listOf(
            findViewById(R.id.btnA), findViewById(R.id.btnB),
            findViewById(R.id.btnC), findViewById(R.id.btnD)
        )
        answerBtns.forEachIndexed { i, b -> b.setOnClickListener { onAnswer(i) } }
        findViewById<Button>(R.id.btnRetry).setOnClickListener { loadQuestions() }
        findViewById<Button>(R.id.btnBack).setOnClickListener { finish() }

        loadQuestions()
    }

    override fun onDestroy() {
        timer?.cancel()
        super.onDestroy()
    }

    private fun loadQuestions() {
        loadingView.visibility = View.VISIBLE
        errorView.visibility = View.GONE
        quizView.visibility = View.GONE
        Thread {
            try {
                if (mode == "trivia") {
                    trivia = Api.fetchTrivia(category, difficulty)
                } else {
                    rounds = buildPokemonRounds()
                }
                runOnUiThread {
                    index = 0; score = 0; correctCount = 0; streak = 0
                    showQuestion()
                }
            } catch (e: Exception) {
                runOnUiThread { showError() }
            }
        }.start()
    }

    private fun buildPokemonRounds(): List<PokemonRound> {
        val names = Api.fetchPokemonNames()
        val usedIds = HashSet<Int>()
        val out = ArrayList<PokemonRound>()
        while (out.size < total) {
            val id = Random.nextInt(1, 1011)
            if (!usedIds.add(id)) continue
            val info = Api.fetchPokemon(id)
            val distractors = names.filter { it != info.name }.shuffled().take(3)
            val choices = (distractors + info.name).shuffled()
            val sprite = info.spriteUrl?.let { Api.downloadBitmap(it) }
            out.add(PokemonRound(info, choices, sprite))
        }
        return out
    }

    private fun showError() {
        loadingView.visibility = View.GONE
        quizView.visibility = View.GONE
        errorView.visibility = View.VISIBLE
        tvError.text = "Butuh internet untuk ambil soal.\nPeriksa koneksi lalu coba lagi."
    }

    private fun showQuestion() {
        loadingView.visibility = View.GONE
        errorView.visibility = View.GONE
        quizView.visibility = View.VISIBLE
        answered = false
        answerBtns.forEach { it.isEnabled = true; it.setBackgroundColor(Color.parseColor("#3A3A3A")) }

        tvProgress.text = "Soal ${index + 1}/$total"
        tvScore.text = "Skor: $score"
        tvStreak.text = if (streak >= 2) "🔥 $streak" else ""

        if (mode == "trivia") {
            val q = trivia[index]
            ivSprite.visibility = View.GONE
            tvHint.visibility = View.GONE
            tvQuestion.text = q.text
            answerBtns.forEachIndexed { i, b -> b.text = q.choices[i] }
        } else {
            val r = rounds[index]
            tvQuestion.text = "Siapa Pokemon ini?"
            if (r.sprite != null) {
                ivSprite.setImageBitmap(r.sprite)
                ivSprite.setColorFilter(Color.BLACK) // siluet
                ivSprite.visibility = View.VISIBLE
            } else {
                ivSprite.visibility = View.GONE
            }
            val h = r.info.heightDm / 10.0
            val w = r.info.weightHg / 10.0
            tvHint.text = "Tipe: ${r.info.types.joinToString(", ")} • Tinggi: ${h}m • Berat: ${w}kg"
            tvHint.visibility = View.VISIBLE
            answerBtns.forEachIndexed { i, b -> b.text = r.choices[i].replaceFirstChar(Char::titlecase) }
        }
        startTimer()
    }

    private fun startTimer() {
        timer?.cancel()
        pbTimer.max = 30
        pbTimer.progress = 30
        tvTimer.text = "30"
        timer = object : CountDownTimer(30000, 1000) {
            override fun onTick(millisUntilFinished: Long) {
                val s = (millisUntilFinished / 1000).toInt()
                pbTimer.progress = s
                tvTimer.text = "$s"
            }
            override fun onFinish() {
                pbTimer.progress = 0
                tvTimer.text = "0"
                onAnswer(-1) // waktu habis = salah
            }
        }.start()
    }

    private fun correctAnswer(): String {
        return if (mode == "trivia") trivia[index].correct else rounds[index].info.name
    }

    private fun onAnswer(picked: Int) {
        if (answered) return
        answered = true
        timer?.cancel()
        val correct = correctAnswer()
        val pickedText = if (picked >= 0) answerBtns[picked].text.toString() else ""
        val isCorrect = picked >= 0 && pickedText.equals(correct, ignoreCase = true)
        if (isCorrect) {
            score += 10
            correctCount++
            streak++
        } else {
            streak = 0
        }
        tvScore.text = "Skor: $score"
        tvStreak.text = if (streak >= 2) "🔥 $streak" else ""
        answerBtns.forEach { b ->
            val t = b.text.toString()
            when {
                t.equals(correct, ignoreCase = true) -> b.setBackgroundColor(Color.parseColor("#2E7D32"))
                picked >= 0 && b == answerBtns[picked] -> b.setBackgroundColor(Color.parseColor("#C62828"))
            }
            b.isEnabled = false
        }
        if (mode == "pokemon") ivSprite.clearColorFilter() // tampilkan warna aslinya
        handler.postDelayed({
            index++
            if (index >= total) finishQuiz() else showQuestion()
        }, 1400)
    }

    private fun finishQuiz() {
        val key = if (mode == "trivia") MainActivity.KEY_TRIVIA else MainActivity.KEY_POKEMON
        val prefs = getSharedPreferences(MainActivity.PREFS, Context.MODE_PRIVATE)
        val oldBest = prefs.getInt(key, 0)
        val isNewBest = score > oldBest
        if (isNewBest) prefs.edit().putInt(key, score).apply()
        val intent = Intent(this, ResultActivity::class.java).apply {
            putExtra("mode", mode)
            putExtra("category", category)
            putExtra("difficulty", difficulty)
            putExtra("score", score)
            putExtra("correct", correctCount)
            putExtra("total", total)
            putExtra("best", maxOf(oldBest, score))
            putExtra("newBest", isNewBest)
        }
        startActivity(intent)
        finish()
    }
}
