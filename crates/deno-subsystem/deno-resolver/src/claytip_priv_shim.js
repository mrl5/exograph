({
    executeQueryPriv: async function (query_string, variables, context_override) {
        const result = await  Deno[Deno.internal].core.opAsync("op_claytip_execute_query_priv", query_string, variables, context_override);
        return result;
    },
})