"""Voice playback wrapper with robust fallback behavior."""

from pathlib import Path
from typing import Any, Dict, Optional

try:
    import pyttsx3 as _pyttsx3
except Exception:
    _pyttsx3 = None
pyttsx3 = _pyttsx3


class VoiceEngine:
    """Primary pyttsx3 playback with optional fallback to tts_module."""

    def __init__(self):

        self.enabled = False

        self.engine = None

        self.voice_id_by_character: Dict[str, str] = {}

        self.default_voice_id: Optional[str] = None

        self._fallback_tts: Any = None
        self._init_failed_permanently = False
        self._export_disabled = False
        self._warned_no_tts_backend = False
        self._init_fallback_backend()

        if pyttsx3 is None:
            return

        self._try_init_engine()

    def _safe_print(self, message: str) -> None:
        """Print warnings without crashing on non-UTF-8 terminals."""

        try:
            print(message)
        except UnicodeEncodeError:
            print(message.encode("ascii", errors="ignore").decode("ascii", errors="ignore"))

    def _init_fallback_backend(self) -> None:
        """Initialize optional fallback TTS backend from tts_module."""

        try:
            from tts_module import get_tts

            self._fallback_tts = get_tts(enabled=True, backend="auto")
        except Exception:
            self._fallback_tts = None

    def _try_init_engine(self) -> bool:
        """Attempt pyttsx3 initialization and voice mapping."""

        if self._init_failed_permanently:
            return False

        if pyttsx3 is None:
            self.engine = None
            self.enabled = False
            return False

        try:
            self.engine = pyttsx3.init()
            self._initialize_character_voices()
            self.enabled = True
            return True
        except Exception as e:
            self._safe_print(f"Voice engine unavailable: {e}")
            self.engine = None
            self.enabled = False
            self._init_failed_permanently = True
            return False

    def _initialize_character_voices(self):
        """Load Windows character voices and map them to known agents."""

        if not self.engine:
            return

        try:
            available_voices = self.engine.getProperty("voices") or []

        except Exception:
            available_voices = []

        male_voice_id = None

        female_voice_id = None

        for voice in available_voices:
            voice_name = (getattr(voice, "name", "") or "").lower()

            if "david" in voice_name and male_voice_id is None:
                male_voice_id = voice.id

            if "zira" in voice_name and female_voice_id is None:
                female_voice_id = voice.id

        current_voice_id = self.engine.getProperty("voice")

        self.default_voice_id = male_voice_id or female_voice_id or current_voice_id

        self.voice_id_by_character = {
            "James": male_voice_id or self.default_voice_id,
            "Luca": male_voice_id or self.default_voice_id,
            "Jasmine": female_voice_id or self.default_voice_id,
            "Elena": female_voice_id or self.default_voice_id,
        }

    def _voice_for_agent(self, agent_name: str) -> Optional[str]:
        """Return mapped voice id for known characters."""

        return self.voice_id_by_character.get(agent_name, self.default_voice_id)

    def speak(self, text: str, agent_name: Optional[str] = None):
        """Speak text synchronously; retry init + fallback when needed."""

        if not text:
            return

        if (not self.enabled or not self.engine) and not self._try_init_engine():
            self._speak_with_fallback(text, agent_name)
            return

        try:
            target_voice = self._voice_for_agent(agent_name or "")

            if target_voice:
                self.engine.setProperty("voice", target_voice)

            self.engine.say(text)

            # Blocks until the queue is empty so audio matches text output order

            self.engine.runAndWait()

        except Exception as e:
            self._safe_print(f"Voice playback failed: {e}")
            self.enabled = False
            self.engine = None
            self._speak_with_fallback(text, agent_name)

    def _speak_with_fallback(self, text: str, agent_name: Optional[str]) -> None:
        """Use tts_module backend when pyttsx3 is unavailable."""

        if self._fallback_tts is None:
            if not self._warned_no_tts_backend:
                self._safe_print("Voice unavailable: no fallback TTS backend is configured.")
                self._warned_no_tts_backend = True
            return

        try:
            self._fallback_tts.speak(text, agent_name=agent_name, async_mode=False)
        except Exception as e:
            self._safe_print(f"Fallback voice playback failed: {e}")

    @staticmethod
    def estimate_duration_ms(text: str) -> int:
        """Estimate speech duration for subtitle timing when no media metadata exists."""

        words = max(1, len(text.split()))
        words_per_minute = 165
        duration_ms = int((words / words_per_minute) * 60_000)
        return max(900, duration_ms)

    def export_to_file(self, text: str, agent_name: Optional[str], output_path: Path) -> bool:
        """Synthesize speech to a local WAV file for external visual clients."""

        if self._export_disabled:
            return False
        if pyttsx3 is None:
            return False
        if not text:
            return False

        try:
            output_path.parent.mkdir(parents=True, exist_ok=True)
            export_engine = pyttsx3.init()

            target_voice = self._voice_for_agent(agent_name or "")
            if target_voice:
                export_engine.setProperty("voice", target_voice)

            export_engine.save_to_file(text, str(output_path))
            export_engine.runAndWait()
            export_engine.stop()

            return output_path.exists() and output_path.stat().st_size > 0

        except Exception as e:
            self._safe_print(f"Voice export failed: {e}")
            self._export_disabled = True

            return False
