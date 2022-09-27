export function add(x, y) {
    return x + y
}

export function divide(x, y) {
    let quotient = Math.floor(x / y);
    let remainder = x % y;

    if (y == 0) {
        throw new ClaytipError("Division by zero is not allowed")
    }

    return {
        "quotient": quotient,
        "remainder": remainder
    }
}

export async function asyncDivide(x, y) {
    return divide(x, y);
}

export function currentUnixEpoch() {
    return Math.floor(Date.now() / 1000)
}

export async function shimQuery(claytip) {
    const result = await claytip.executeQuery(
        `query {
            foos(where: {baz: {eq: 4}}) {
                id
            }
        }`
    );

    let str = "The `foos` with `baz` = 4 have IDs of ";

    for (const foo of result.foos) {
        str += foo.id += ", ";
    }

    return str;
}

export function testMutation(claytip) {
    return 3.14
}

export function illegalFunction() {
    const x = undefined;
    return x[0]
}

export function log(env, message) {
    console.log(message)
    return true
}