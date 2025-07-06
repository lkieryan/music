export const appLog = (...args: any[]) => {
  // if (ELECTRON) log(...args)
  console.info(
    "Music",
    "color: #fff; margin: 0; padding: 5px 0; background: #ff5c00; border-radius: 3px;",
    ...args.reduce((acc, cur) => {
      acc.push("", cur)
      return acc
    }, []),
  )
}
