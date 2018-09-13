# **zbot v0.2.6**

--------------------------------------------------------------------------------

## Permissions
- `r`: `ReadOnly`
- `o`: `Owners`
- `b`: `Broadcaster`
- `m`: `Moderators`
- `s`: `Subscribers`
- `v`: `Viewers`

There are two prefixes: `+`/`-`. The `+` allows the specified groups to use the command, while `-` denies that ability.
For example, to change the permissions of a built-in command,  alias it to itself: `!alias quoteadd +v -r quoteadd`.
Here, `+v` allows all viewers to use the command (potentially dangerous!), and `-r` allows arguments to be provided to the alias.

**Note**: Multiple groups may be combined into a single argument, e.g. `+bms`.

**Note**: `ReadOnly` determines whether or not an alias will accept arguments. It is set by default on all aliases for security reasons.

--------------------------------------------------------------------------------

## Commands

### QuoteDB
- `!quote`: Prints the quote with the specified id, otherwise if no id is specified a random quote is printed.
    - **Permissions**: `Broadcaster`, `Mods`
    - **Usage**: `!quote [id]`
    - **Example**: `!quote 3`
- `!quoteadd`: Adds a quote to the QuoteDB.
    - **Permissions**: `Broadcaster`, `Mods`
    - **Usage**: `!quoteadd <quote text>`
    - **Example**: `!quoteadd "Hello, world!" - ZedExV, 2018`
- `!quoterm`: Removes a quote by id from the QuoteDB
    - **Permissions**: `Broadcaster`, `Mods`
    - **Usage**: `!quoterm <id>`
    - **Example**: `!quoterm 3`

### Utility Commands
- `!strawpoll`: Create a new [strawpoll](https://strawpoll.me/) or query its results.
    - **Permissions**: `Broadcaster`, `Mods`
    - **Usage**: `!strawpoll [<title> | <Option 1> | <Option 2> | ... | [Option N]]`
    - **Example**: `!strawpoll Is Zed human? | Yes | Yes | Yes`
    - **Example**: `!stawpoll`
    - **Note**: Strawpoll requires a minimum of 2 options and will accept a maximum of 30 options.
    - **Note**: When the `!strawpoll` command is invoked without arguments the results for the last poll will be printed to chat.

### RNG Commands
- `!8ball`: Ask the all knowing 8ball a question!
    - **Permissions**: `Viewers`
    - **Usage**: `!8ball [question]`
    - **Example**: `!8ball should I sub to ZedExV?`
- `!flipcoin`: Without any arguments, one coin is flipped and the result is printed. If a numeric argument is provided, that number of coins is flipped and the result printed.
    - **Permissions**: `Viewers`
    - **Usage**: `!flipcoin [N]`
    - **Example**: `!flipcoin 7`
- `!roll`: Rolls one or several die. Without any arguments, a single d20 is rolled.
    - **Permissions**: `Viewers`
    - **Usage**: `!roll [[X]dY + ... [<+|-> Z]]`
    - **Example**: `!roll 2d20 + d7 - 4` rolls 2 20-sided die and a 7-sided die, then subtracts 4 from the result.
    - **Note**: You can use [AnyDice](https://anydice.com/) to check the resulting distribution.

### Translation Commands
- `!thicc`: Converts text into the 'thicc' alphabet (latin alphabet only).
    - **Permissions**: `Viewers`
    - **Usage**: `!thicc <text>`
    - **Example**: `!thicc Hello, world!`
- `!tiny`: Convert text into its tiny equivalent (latin alphabet only).
    - **Permissions**: `Viewers`
    - **Usage**: `!tiny <text>`
    - **Example**: `!tiny Hello, world!`
- `!smol`: Convert text into its small caps equivalent (latin alphabet only).
    - **Permissions**: `Viewers`
    - **Usage**: `!smol <text>`
    - **Example**: `!smol Hello, world!`

### Meme Commands
- `!numberwang`: Determine if a number is numberwang or not!
    - **Permissions**: `Viewers`
    - **Usage**: `!numberwang <N>`
    - **Example**: `!numberwang 420.69`
- `!tcount`: Determine your personal T-count (out of 100).
    - **Permissions**: `Viewers`
    - **Usage**: `!tcount`

### Special Commands
- `!alias`: Alias a command to another command. Only someone with the ability to use to aliased command can create an alias to that command.
    - **Permissions**: `Broadcaster`, `Mods`
    - **Usage**: `!alias <alias> [permissions...] <command> [args...]`
    - **Example**: `!alias tcount null` blacklists `!tcount`.
    - **Example**: `!alias quoteadd +v -r quoteadd` allows viewers to add quotes.
    - **Example**: `!alias discord +v say discord.gg/XXXXXXX` prints 'discord.gg/XXXXXXX' when !discord is invoked.
    - **Note**: You can only alias built-in commands.
    - **Note**: Aliases copy the permissions of the command they are aliasing.
- `!aliasmod`: Change permissions for an alias.
    - Permissions `Broadcaster`, `Mods`
    - **Usage**: `!aliasmod <alias> <permissions...>`
    - **Example**: `!aliasmod quoteadd +s` allows subs to use the quoteadd alias.
    - **Note**: The alias must already exist to modify the permissions, and permissions may not be directly modified for built-in commands (they must have an alias to theirself).
- `!say`: Sends a message to the chat.
    - **Permissions**: `Broadcaster`, `Mods`
    - **Usage**: `!say <message>`
    - **Example**: `!say Hello, world!`
    - **Note**: Allowing viewers to use this command with the ReadOnly permission disabled is **dangerous**. If the bot is modded, they would be able to vicariously use mod commands.
- `!null`: Does absolutely nothing. This is used to blacklist commands.
    - **Permissions**: `Broadcaster`
- `!version`: Print bot version information.
    - **Permissions**: `Owners`
    - **Usage**: `!version`
- `!shutdown`: Causes the bot to shutdown.
    - **Permissions**: `Owners`
    - **Usage**: `!shutdown`
