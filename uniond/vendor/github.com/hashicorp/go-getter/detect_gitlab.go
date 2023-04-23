package getter

import (
	"fmt"
	"net/url"
	"strings"
)

// GitLabDetector implements Detector to detect GitLab URLs and turn
// them into URLs that the Git Getter can understand.
type GitLabDetector struct{}

func (d *GitLabDetector) Detect(src, _ string) (string, bool, error) {
	if len(src) == 0 {
		return "", false, nil
	}

	if strings.HasPrefix(src, "gitlab.com/") {
		return d.detectHTTP(src)
	}

	return "", false, nil
}

func (d *GitLabDetector) detectHTTP(src string) (string, bool, error) {
	parts := strings.Split(src, "/")
	if len(parts) < 3 {
		return "", false, fmt.Errorf(
			"GitLab URLs should be gitlab.com/username/repo")
	}

	urlStr := fmt.Sprintf("https://%s", strings.Join(parts[:3], "/"))
	repoUrl, err := url.Parse(urlStr)
	if err != nil {
		return "", true, fmt.Errorf("error parsing GitLab URL: %s", err)
	}

	if !strings.HasSuffix(repoUrl.Path, ".git") {
		repoUrl.Path += ".git"
	}

	if len(parts) > 3 {
		repoUrl.Path += "//" + strings.Join(parts[3:], "/")
	}

	return "git::" + repoUrl.String(), true, nil
}
