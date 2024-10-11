package test

import (
	"context"
	"fmt"

	"google.golang.org/grpc/metadata"
)

type metaContext string

const (
	correlationID     metaContext = "correlation-id"
	responseHDR       metaContext = "my-response-header"
	responseTRLR      metaContext = "my-response-trailer"
	correlationIDTRLR metaContext = "correlation-id-consumed"
)

/* client before functions */

func injectCorrelationID(ctx context.Context, md *metadata.MD) context.Context {
	if hdr, ok := ctx.Value(correlationID).(string); ok {
		fmt.Printf("\tClient found correlationID %q in context, set metadata header\n", hdr)
		(*md)[string(correlationID)] = append((*md)[string(correlationID)], hdr)
	}
	return ctx
}

func displayClientRequestHeaders(ctx context.Context, md *metadata.MD) context.Context {
	if len(*md) > 0 {
		fmt.Println("\tClient >> Request Headers:")
		for key, val := range *md {
			fmt.Printf("\t\t%s: %s\n", key, val[len(val)-1])
		}
	}
	return ctx
}

/* server before functions */

func extractCorrelationID(ctx context.Context, md metadata.MD) context.Context {
	if hdr, ok := md[string(correlationID)]; ok {
		cID := hdr[len(hdr)-1]
		ctx = context.WithValue(ctx, correlationID, cID)
		fmt.Printf("\tServer received correlationID %q in metadata header, set context\n", cID)
	}
	return ctx
}

func displayServerRequestHeaders(ctx context.Context, md metadata.MD) context.Context {
	if len(md) > 0 {
		fmt.Println("\tServer << Request Headers:")
		for key, val := range md {
			fmt.Printf("\t\t%s: %s\n", key, val[len(val)-1])
		}
	}
	return ctx
}

/* server after functions */

func injectResponseHeader(ctx context.Context, md *metadata.MD, _ *metadata.MD) context.Context {
	*md = metadata.Join(*md, metadata.Pairs(string(responseHDR), "has-a-value"))
	return ctx
}

func displayServerResponseHeaders(ctx context.Context, md *metadata.MD, _ *metadata.MD) context.Context {
	if len(*md) > 0 {
		fmt.Println("\tServer >> Response Headers:")
		for key, val := range *md {
			fmt.Printf("\t\t%s: %s\n", key, val[len(val)-1])
		}
	}
	return ctx
}

func injectResponseTrailer(ctx context.Context, _ *metadata.MD, md *metadata.MD) context.Context {
	*md = metadata.Join(*md, metadata.Pairs(string(responseTRLR), "has-a-value-too"))
	return ctx
}

func injectConsumedCorrelationID(ctx context.Context, _ *metadata.MD, md *metadata.MD) context.Context {
	if hdr, ok := ctx.Value(correlationID).(string); ok {
		fmt.Printf("\tServer found correlationID %q in context, set consumed trailer\n", hdr)
		*md = metadata.Join(*md, metadata.Pairs(string(correlationIDTRLR), hdr))
	}
	return ctx
}

func displayServerResponseTrailers(ctx context.Context, _ *metadata.MD, md *metadata.MD) context.Context {
	if len(*md) > 0 {
		fmt.Println("\tServer >> Response Trailers:")
		for key, val := range *md {
			fmt.Printf("\t\t%s: %s\n", key, val[len(val)-1])
		}
	}
	return ctx
}

/* client after functions */

func displayClientResponseHeaders(ctx context.Context, md metadata.MD, _ metadata.MD) context.Context {
	if len(md) > 0 {
		fmt.Println("\tClient << Response Headers:")
		for key, val := range md {
			fmt.Printf("\t\t%s: %s\n", key, val[len(val)-1])
		}
	}
	return ctx
}

func displayClientResponseTrailers(ctx context.Context, _ metadata.MD, md metadata.MD) context.Context {
	if len(md) > 0 {
		fmt.Println("\tClient << Response Trailers:")
		for key, val := range md {
			fmt.Printf("\t\t%s: %s\n", key, val[len(val)-1])
		}
	}
	return ctx
}

func extractConsumedCorrelationID(ctx context.Context, _ metadata.MD, md metadata.MD) context.Context {
	if hdr, ok := md[string(correlationIDTRLR)]; ok {
		fmt.Printf("\tClient received consumed correlationID %q in metadata trailer, set context\n", hdr[len(hdr)-1])
		ctx = context.WithValue(ctx, correlationIDTRLR, hdr[len(hdr)-1])
	}
	return ctx
}

/* CorrelationID context handlers */

func SetCorrelationID(ctx context.Context, v string) context.Context {
	return context.WithValue(ctx, correlationID, v)
}

func GetConsumedCorrelationID(ctx context.Context) string {
	if trlr, ok := ctx.Value(correlationIDTRLR).(string); ok {
		return trlr
	}
	return ""
}
