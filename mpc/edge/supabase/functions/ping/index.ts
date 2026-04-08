const resendSecret = Deno.env.get("RESEND_SECRET")
const resendApiKey = Deno.env.get("RESEND_API_KEY")

// Timing-safe comparison to prevent timing-based secret oracle attacks
function timingSafeEqual(a: string, b: string): boolean {
  const encoder = new TextEncoder()
  const aBytes = encoder.encode(a)
  const bBytes = encoder.encode(b)
  if (aBytes.length !== bBytes.length) {
    // Still run the comparison on equal-length buffers to avoid length oracle
    crypto.subtle.timingSafeEqual(aBytes, aBytes)
    return false
  }
  return crypto.subtle.timingSafeEqual(aBytes, bBytes)
}

const handler = async (request: Request): Promise<Response> => {
  // Validate secret from Authorization header rather than the request body,
  // so it is never stored in request-body logs or application traces.
  const authHeader = request.headers.get("Authorization")
  const providedSecret =
    authHeader?.startsWith("Bearer ") ? authHeader.slice(7) : null

  if (!resendSecret || !providedSecret || !timingSafeEqual(providedSecret, resendSecret)) {
    return new Response(JSON.stringify("too bad"), {
      status: 403,
      headers: {
        "Content-Type": "application/json",
      },
    })
  }

  const { email } = await request.json()
  const res = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": `Bearer ${resendApiKey}`,
    },
    body: JSON.stringify({
      from: "Union Ceremony <alert@ceremony.union.build>",
      to: [email],
      reply_to: "ceremony@union.build",
      subject: "Your Turn Is Almost Here - Log Into Union Ceremony",
      html: `
        <p>
          Your contribution slot for the Union Trusted Setup Ceremony is almost here.
        </p>
        <p>
          Your place in queue: <strong>5</strong><br/>
          Estimated time until your slot: between <strong>1 hour</strong> and <strong>5 hours</strong>
        </p>
        <p>
          Please go to <strong><a href="https://ceremony.union.build">ceremony.union.build</a></strong>, log in, and follow all steps on the page.<br/>
          If you do not follow all steps by the time your contribution slot arrives, <strong>you will lose your slot</strong>.
        </p>
      `,
    }),
  })
  const data = await res.json()
  return new Response(JSON.stringify(data), {
    status: 200,
    headers: {
      "Content-Type": "application/json",
    },
  })
}

Deno.serve(handler)
