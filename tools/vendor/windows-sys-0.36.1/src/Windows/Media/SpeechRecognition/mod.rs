pub type ISpeechRecognitionConstraint = *mut ::core::ffi::c_void;
pub type SpeechContinuousRecognitionCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechContinuousRecognitionMode(pub i32);
impl SpeechContinuousRecognitionMode {
    pub const Default: Self = Self(0i32);
    pub const PauseOnRecognition: Self = Self(1i32);
}
impl ::core::marker::Copy for SpeechContinuousRecognitionMode {}
impl ::core::clone::Clone for SpeechContinuousRecognitionMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpeechContinuousRecognitionResultGeneratedEventArgs = *mut ::core::ffi::c_void;
pub type SpeechContinuousRecognitionSession = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechRecognitionAudioProblem(pub i32);
impl SpeechRecognitionAudioProblem {
    pub const None: Self = Self(0i32);
    pub const TooNoisy: Self = Self(1i32);
    pub const NoSignal: Self = Self(2i32);
    pub const TooLoud: Self = Self(3i32);
    pub const TooQuiet: Self = Self(4i32);
    pub const TooFast: Self = Self(5i32);
    pub const TooSlow: Self = Self(6i32);
}
impl ::core::marker::Copy for SpeechRecognitionAudioProblem {}
impl ::core::clone::Clone for SpeechRecognitionAudioProblem {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpeechRecognitionCompilationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechRecognitionConfidence(pub i32);
impl SpeechRecognitionConfidence {
    pub const High: Self = Self(0i32);
    pub const Medium: Self = Self(1i32);
    pub const Low: Self = Self(2i32);
    pub const Rejected: Self = Self(3i32);
}
impl ::core::marker::Copy for SpeechRecognitionConfidence {}
impl ::core::clone::Clone for SpeechRecognitionConfidence {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechRecognitionConstraintProbability(pub i32);
impl SpeechRecognitionConstraintProbability {
    pub const Default: Self = Self(0i32);
    pub const Min: Self = Self(1i32);
    pub const Max: Self = Self(2i32);
}
impl ::core::marker::Copy for SpeechRecognitionConstraintProbability {}
impl ::core::clone::Clone for SpeechRecognitionConstraintProbability {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechRecognitionConstraintType(pub i32);
impl SpeechRecognitionConstraintType {
    pub const Topic: Self = Self(0i32);
    pub const List: Self = Self(1i32);
    pub const Grammar: Self = Self(2i32);
    pub const VoiceCommandDefinition: Self = Self(3i32);
}
impl ::core::marker::Copy for SpeechRecognitionConstraintType {}
impl ::core::clone::Clone for SpeechRecognitionConstraintType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpeechRecognitionGrammarFileConstraint = *mut ::core::ffi::c_void;
pub type SpeechRecognitionHypothesis = *mut ::core::ffi::c_void;
pub type SpeechRecognitionHypothesisGeneratedEventArgs = *mut ::core::ffi::c_void;
pub type SpeechRecognitionListConstraint = *mut ::core::ffi::c_void;
pub type SpeechRecognitionQualityDegradingEventArgs = *mut ::core::ffi::c_void;
pub type SpeechRecognitionResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechRecognitionResultStatus(pub i32);
impl SpeechRecognitionResultStatus {
    pub const Success: Self = Self(0i32);
    pub const TopicLanguageNotSupported: Self = Self(1i32);
    pub const GrammarLanguageMismatch: Self = Self(2i32);
    pub const GrammarCompilationFailure: Self = Self(3i32);
    pub const AudioQualityFailure: Self = Self(4i32);
    pub const UserCanceled: Self = Self(5i32);
    pub const Unknown: Self = Self(6i32);
    pub const TimeoutExceeded: Self = Self(7i32);
    pub const PauseLimitExceeded: Self = Self(8i32);
    pub const NetworkFailure: Self = Self(9i32);
    pub const MicrophoneUnavailable: Self = Self(10i32);
}
impl ::core::marker::Copy for SpeechRecognitionResultStatus {}
impl ::core::clone::Clone for SpeechRecognitionResultStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechRecognitionScenario(pub i32);
impl SpeechRecognitionScenario {
    pub const WebSearch: Self = Self(0i32);
    pub const Dictation: Self = Self(1i32);
    pub const FormFilling: Self = Self(2i32);
}
impl ::core::marker::Copy for SpeechRecognitionScenario {}
impl ::core::clone::Clone for SpeechRecognitionScenario {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpeechRecognitionSemanticInterpretation = *mut ::core::ffi::c_void;
pub type SpeechRecognitionTopicConstraint = *mut ::core::ffi::c_void;
pub type SpeechRecognitionVoiceCommandDefinitionConstraint = *mut ::core::ffi::c_void;
pub type SpeechRecognizer = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Media_SpeechRecognition\"`*"]
#[repr(transparent)]
pub struct SpeechRecognizerState(pub i32);
impl SpeechRecognizerState {
    pub const Idle: Self = Self(0i32);
    pub const Capturing: Self = Self(1i32);
    pub const Processing: Self = Self(2i32);
    pub const SoundStarted: Self = Self(3i32);
    pub const SoundEnded: Self = Self(4i32);
    pub const SpeechDetected: Self = Self(5i32);
    pub const Paused: Self = Self(6i32);
}
impl ::core::marker::Copy for SpeechRecognizerState {}
impl ::core::clone::Clone for SpeechRecognizerState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SpeechRecognizerStateChangedEventArgs = *mut ::core::ffi::c_void;
pub type SpeechRecognizerTimeouts = *mut ::core::ffi::c_void;
pub type SpeechRecognizerUIOptions = *mut ::core::ffi::c_void;
pub type VoiceCommandSet = *mut ::core::ffi::c_void;
