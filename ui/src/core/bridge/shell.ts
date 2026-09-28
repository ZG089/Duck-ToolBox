/** POSIX single-quote quoting: the only shell quoting that needs no further escaping. */
export function shellQuote(value: string): string {
  return `'${value.replaceAll("'", `'\\''`)}'`
}

/**
 * Pipes `input` into `command` through a quoted here-document, so the payload is never
 * interpreted by the shell. The delimiter is random and checked against the payload.
 */
export function heredoc(command: string, input: string): string {
  let marker = ""
  do {
    marker = `DUCK_EOF_${Math.random().toString(36).slice(2, 10).toUpperCase()}`
  } while (input.includes(marker))
  return `${command} <<'${marker}'\n${input}\n${marker}`
}
