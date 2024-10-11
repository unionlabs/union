package backup

import (
	"context"
	"fmt"
	"net/http"

	"github.com/sirupsen/logrus"
)

// Exporter defines a backup exporter methods.
type Exporter interface {
	Backup(ctx context.Context, outputPath string, permissionOverride bool) error
}

// Handler for accepting requests to initiate a new database backup.
func Handler(bk Exporter, outputDir string) func(http.ResponseWriter, *http.Request) {
	log := logrus.WithField("prefix", "db")

	return func(w http.ResponseWriter, r *http.Request) {
		log.Debug("Creating database backup from HTTP webhook")

		_, permissionOverride := r.URL.Query()["permissionOverride"]

		if err := bk.Backup(context.Background(), outputDir, permissionOverride); err != nil {
			log.WithError(err).Error("Failed to create backup")
			w.WriteHeader(http.StatusInternalServerError)
			return
		}
		w.WriteHeader(http.StatusOK)
		_, err := fmt.Fprint(w, "OK")
		if err != nil {
			log.WithError(err).Error("Failed to write OK")
		}
	}
}
