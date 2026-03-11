import sys
import os
from PySide6.QtWidgets import (QDialog, QVBoxLayout, QHBoxLayout, QLabel, 
                             QPushButton, QStackedWidget, QWidget, QComboBox, 
                             QRadioButton, QButtonGroup, QFrame)
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont, QColor
from ui.settings_window import KeyRecorder

class SetupGuide(QDialog):
    finished_setup = Signal()

    def __init__(self, config_manager):
        super().__init__()
        self.config = config_manager
        self.setWindowTitle(self.config.t("setup_title"))
        self.setMinimumSize(800, 600)
        self.setStyleSheet("background-color: #0f0f0f; color: white;")
        self.setWindowFlags(Qt.Window | Qt.WindowTitleHint | Qt.CustomizeWindowHint)

        self.layout = QVBoxLayout(self)
        self.stack = QStackedWidget()
        
        # Step 1: Language
        self.stack.addWidget(self.create_lang_step())
        # Step 2: Model
        self.stack.addWidget(self.create_model_step())
        # Step 3: Hotkey
        self.stack.addWidget(self.create_hotkey_step())
        
        self.layout.addWidget(self.stack)
        
        # Navigation
        self.nav_layout = QHBoxLayout()
        self.btn_next = QPushButton(self.config.t("next"))
        self.btn_next.setFixedSize(150, 50)
        self.btn_next.setStyleSheet("background: #26a69a; border-radius: 10px; font-weight: bold; font-size: 16px;")
        self.btn_next.clicked.connect(self.next_step)
        self.nav_layout.addStretch()
        self.nav_layout.addWidget(self.btn_next)
        self.layout.addLayout(self.nav_layout)

    def next_step(self):
        idx = self.stack.currentIndex()
        if idx < self.stack.count() - 1:
            self.stack.setCurrentIndex(idx + 1)
            if idx + 1 == self.stack.count() - 1:
                self.btn_next.setText(self.config.t("finish"))
        else:
            self.config.set("first_run", False)
            self.accept()
            self.finished_setup.emit()

    def create_lang_step(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(50, 50, 50, 50)
        lay.setSpacing(30)
        
        title = QLabel("Select Language / Sprache wählen")
        title.setStyleSheet("font-size: 32px; font-weight: bold;")
        lay.addWidget(title, alignment=Qt.AlignCenter)
        
        self.lang_combo = QComboBox()
        self.lang_combo.addItems(["English", "Deutsch"])
        self.lang_combo.setFixedSize(300, 50)
        self.lang_combo.setStyleSheet("background: #1a1a1a; border: 1px solid #333; padding: 10px; font-size: 18px;")
        self.lang_combo.currentTextChanged.connect(self.update_lang)
        lay.addWidget(self.lang_combo, alignment=Qt.AlignCenter)
        return page

    def update_lang(self, text):
        lang = "de" if text == "Deutsch" else "en"
        self.config.set("language", lang)
        self.btn_next.setText(self.config.t("next"))

    def create_model_step(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(50, 50, 50, 50)
        lay.setSpacing(20)
        
        title = QLabel(self.config.t("model_label"))
        title.setStyleSheet("font-size: 28px; font-weight: bold;")
        lay.addWidget(title, alignment=Qt.AlignCenter)
        
        models = [
            ("tiny", "Very Fast - Basic accuracy"),
            ("base", "Fast - Good for simple sentences"),
            ("small", "Balanced - Recommended for most"),
            ("medium", "High Quality - Professional level"),
            ("large-v3", "Extreme - Maximum accuracy (needs more RAM)")
        ]
        
        self.model_group = QButtonGroup(self)
        for code, desc in models:
            rb = QRadioButton(f"{code.upper()} ({desc})")
            rb.setStyleSheet("font-size: 18px; padding: 10px;")
            if code == self.config.get("model_size"): rb.setChecked(True)
            self.model_group.addButton(rb)
            rb.toggled.connect(lambda checked, c=code: self.config.set("model_size", c) if checked else None)
            lay.addWidget(rb)
            
        return page

    def create_hotkey_step(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(50, 50, 50, 50)
        lay.setSpacing(30)
        
        title = QLabel(self.config.t("hotkey_label"))
        title.setStyleSheet("font-size: 28px; font-weight: bold;")
        lay.addWidget(title, alignment=Qt.AlignCenter)
        
        self.recorder = KeyRecorder(self.config.get("hotkey"))
        self.recorder.key_recorded.connect(lambda k: self.config.set("hotkey", k))
        lay.addWidget(self.recorder, alignment=Qt.AlignCenter)
        
        hint = QLabel(self.config.t("hotkey_hint"))
        hint.setStyleSheet("color: #666; font-size: 16px;")
        lay.addWidget(hint, alignment=Qt.AlignCenter)
        
        return page
