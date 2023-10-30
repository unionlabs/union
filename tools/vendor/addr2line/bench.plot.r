v <- read.table(file("stdin"))
t <- data.frame(prog=v[,1], funcs=(v[,2]=="func"), time=v[,3], mem=v[,4], stringsAsFactors=FALSE)

t$prog <- as.character(t$prog)
t$prog[t$prog == "master"] <- "gimli-rs/addr2line"
t$funcs[t$funcs == TRUE] <- "With functions"
t$funcs[t$funcs == FALSE] <- "File/line only"
t$mem = t$mem / 1024.0

library(ggplot2)
p <- ggplot(data=t, aes(x=prog, y=time, fill=prog))
p <- p + geom_bar(stat = "identity")
p <- p + facet_wrap(~ funcs)
p <- p + theme(axis.title.x=element_blank(), axis.text.x=element_blank(), axis.ticks.x=element_blank())
p <- p + ylab("time (s)") + ggtitle("addr2line runtime")
ggsave('time.png',plot=p,width=10,height=6)

p <- ggplot(data=t, aes(x=prog, y=mem, fill=prog))
p <- p + geom_bar(stat = "identity")
p <- p + facet_wrap(~ funcs)
p <- p + theme(axis.title.x=element_blank(), axis.text.x=element_blank(), axis.ticks.x=element_blank())
p <- p + ylab("memory (kB)") + ggtitle("addr2line memory usage")
ggsave('memory.png',plot=p,width=10,height=6)
