window.fetchFromUrl = async function fetchFromUrl(url) {
            try {
                const response = await fetch(url);
                if (!response.ok) {
                    throw new Error(`HTTP error! status: ${response.status}`);
                }
                return await response.text();
            } catch (error) {
                console.error('Error fetching SM chart:', error);
                throw error;
            }
        }

      window.measureTime = async function measureTime(func, ...args) {
    const func_name = func.name;
    const start = performance.now();
    try {
        return await func(...args);
    } catch (err) {
        const contextError = new Error(`[${func_name}] ${err.message}`);
        throw contextError;
    } finally {
        const end = performance.now();
        const duration = (end - start).toFixed(3);
        console.log(`${func_name} took: ${duration} milliseconds`);
    }
}
        
initWasm().catch(err => console.error("Failed to initialize WASM:", err));

document.getElementById('testJs').addEventListener('click', () => runJsTests());

async function initWasm() {
    try {
        const wasm = await import('./bin/rgc_chart.js');
        wasm.default();
        window.wasm = await wasm;
        console.log("Wasm initialized successfully!");
    } catch (err) {
        console.error("Wasm init failed:", err);
        throw err;
    }
}


async function runJsTests() {
    try {
        const to_osu = await import('./js/to_osu.js');
        const to_sm = await import('./js/to_sm.js');
        const to_qua = await import('./js/to_qua.js');


        await measureTime(to_osu.sm_to_osu,
            await fetchFromUrl('/Maps/etterna/Kil_ChineseTea/ct.sm'))
            .catch(err => console.error(err));
        await measureTime(to_osu.qua_to_osu,
            await fetchFromUrl('/Maps/quaver/4548_886_Ziqqurat/34785.qua'))
            .catch(err => console.error(err));
        
        await measureTime(to_sm.osu_to_sm,
            await fetchFromUrl('/Maps/osu/1688622_EverGreen/ClumsyRecord - Ever Green feat. Ganeme (FAMoss) [Misfortune Lunatic].osu'))
            .catch(err => console.error(err));
        await measureTime(to_sm.qua_to_sm,
            await fetchFromUrl('/Maps/quaver/4548_886_Ziqqurat/34785.qua'))
            .catch(err => console.error(err));

        await measureTime(to_qua.osu_to_qua,
            await fetchFromUrl('/Maps/osu/360565_HatsuneMikuNoShoushitsu/cosMo@BousouP feat. Hatsune Miku - Hatsune Miku no Shoushitsu (juankristal) [Disappearance].osu'))
            .catch(err => console.error(err));
        await measureTime(to_qua.sm_to_qua,
            await fetchFromUrl('/Maps/etterna/Kil_ChineseTea/ct.sm'))
            .catch(err => console.error(err));
        
    } catch (err) {
        console.error("JS test initialization error:", err);
    }
}