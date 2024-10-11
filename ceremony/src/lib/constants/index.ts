export const AXIOM_KEY = "xaat-c2ba1f52-0af4-4814-81d1-996c285912c8"
export const COMMAND =
  "mkdir -p ceremony && docker pull ghcr.io/unionlabs/union/mpc-client:v1.1 && docker run -v $(pwd)/ceremony:/ceremony -w /ceremony -p 4919:4919 --rm -it ghcr.io/unionlabs/union/mpc-client:v1.1"
