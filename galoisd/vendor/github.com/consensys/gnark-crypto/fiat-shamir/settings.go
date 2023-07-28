package fiatshamir

import "hash"

type Settings struct {
	Transcript     *Transcript
	Prefix         string
	BaseChallenges [][]byte
	Hash           hash.Hash
}

func WithTranscript(transcript *Transcript, prefix string, baseChallenges ...[]byte) Settings {
	return Settings{
		Transcript:     transcript,
		Prefix:         prefix,
		BaseChallenges: baseChallenges,
	}
}

func WithHash(hash hash.Hash, baseChallenges ...[]byte) Settings {
	return Settings{
		BaseChallenges: baseChallenges,
		Hash:           hash,
	}
}
