/* ==========================================================================
   UloOS Web Application Script - Interactive Boot & Copy Tools
   ========================================================================== */

document.addEventListener('DOMContentLoaded', () => {
    // 1. Terminal Boot Timeline Simulator
    const startBootBtn = document.getElementById('start-boot-btn');
    const resetBootBtn = document.getElementById('reset-boot-btn');
    const bootTerminal = document.getElementById('boot-terminal');
    const mockDesktop = document.getElementById('mock-desktop');
    const logContainer = document.getElementById('terminal-log');
    
    // Web Audio API Retro Startup Synthesizer
    function playRetroChime() {
        try {
            const AudioContext = window.AudioContext || window.webkitAudioContext;
            if (!AudioContext) return;
            const ctx = new AudioContext();
            
            // Nostalgic retro dual-tone ascending bell beep
            const now = ctx.currentTime;
            
            // Tone 1
            const osc1 = ctx.createOscillator();
            const gain1 = ctx.createGain();
            osc1.type = 'triangle';
            osc1.frequency.setValueAtTime(523.25, now); // C5
            osc1.frequency.exponentialRampToValueAtTime(783.99, now + 0.25); // G5
            osc1.frequency.exponentialRampToValueAtTime(1046.50, now + 0.6); // C6
            
            gain1.gain.setValueAtTime(0.15, now);
            gain1.gain.exponentialRampToValueAtTime(0.01, now + 0.8);
            
            osc1.connect(gain1);
            gain1.connect(ctx.destination);
            osc1.start(now);
            osc1.stop(now + 0.8);

            // Tone 2 (Harmonic minor delay)
            setTimeout(() => {
                const osc2 = ctx.createOscillator();
                const gain2 = ctx.createGain();
                osc2.type = 'sine';
                osc2.frequency.setValueAtTime(659.25, ctx.currentTime); // E5
                osc2.frequency.exponentialRampToValueAtTime(1318.51, ctx.currentTime + 0.5); // E6
                
                gain2.gain.setValueAtTime(0.1, ctx.currentTime);
                gain2.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.7);
                
                osc2.connect(gain2);
                gain2.connect(ctx.destination);
                osc2.start(ctx.currentTime);
                osc2.stop(ctx.currentTime + 0.7);
            }, 150);

        } catch (e) {
            console.log("Audio synthesis blocked or unsupported:", e);
        }
    }

    const bootMessages = [
        { text: "Loading UloOS Kernel static variables...", delay: 200 },
        { text: "Initializing GDT & IDT interrupt tables...", delay: 300 },
        { text: "Allocating 64KB static BACKBUFFER structures...", delay: 200 },
        { text: "Initializing VGA Mode 13h (320x200 graphics)... [SUCCESS]", delay: 250 },
        { text: "Probing auxiliary devices: PS/2 Keyboard... Found.", delay: 200 },
        { text: "Probing auxiliary devices: PS/2 Mouse... Found.", delay: 150 },
        { text: "Setting hardware sensitivity: resolution=8counts/mm... OK.", delay: 200 },
        { text: "Setting hardware polling speed: sample_rate=200Hz... OK.", delay: 200 },
        { text: "Registering double-buffer render pointers...", delay: 150 },
        { text: "Loading Startup sound assets... Completed.", delay: 150 },
        { text: "Ready! Booting into Graphical Shell...", delay: 300 }
    ];

    async function executeBootAnimation() {
        startBootBtn.disabled = true;
        startBootBtn.innerHTML = `<i class="fa-solid fa-spinner fa-spin"></i> Booting...`;
        
        // Append initial spacer
        const p1 = document.createElement('p');
        p1.className = 'text-blue';
        p1.innerText = ">>> Starting UloOS Kernel Runtime Engine...";
        logContainer.appendChild(p1);
        logContainer.scrollTop = logContainer.scrollHeight;

        for (const msg of bootMessages) {
            await new Promise(resolve => setTimeout(resolve, msg.delay));
            const logLine = document.createElement('p');
            logLine.innerText = `[OK] ${msg.text}`;
            logContainer.appendChild(logLine);
            logContainer.scrollTop = logContainer.scrollHeight;
        }

        // Play startup beep sound
        playRetroChime();

        // Fade out terminal, Fade in mock desktop
        await new Promise(resolve => setTimeout(resolve, 600));
        bootTerminal.classList.add('hidden');
        mockDesktop.classList.remove('hidden');
    }

    function rebootSystem() {
        // Reset terminal contents
        logContainer.innerHTML = `
            <p class="text-dim">UloOS MBR Loader v1.0.0...</p>
            <p class="text-dim">Reading sector 2 to 32... Success.</p>
            <p class="text-dim">Switching to x86_64 long mode... Enabled.</p>
            <p class="text-dim">Press BOOT to initiate hardware & render kernel...</p>
        `;
        startBootBtn.disabled = false;
        startBootBtn.innerHTML = `<i class="fa-solid fa-power-off"></i> BOOT UloOS`;
        
        // Toggle view visibility
        mockDesktop.classList.add('hidden');
        bootTerminal.classList.remove('hidden');
    }

    startBootBtn.addEventListener('click', executeBootAnimation);
    resetBootBtn.addEventListener('click', rebootSystem);

    // 2. Interactive Mock Mouse Cursor Movement inside Visual monitor
    const monitorScreen = document.querySelector('.desktop-workspace');
    const mockCursor = document.getElementById('mock-cursor');

    monitorScreen.addEventListener('mousemove', (e) => {
        if (mockDesktop.classList.contains('hidden')) return;
        
        const rect = monitorScreen.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        // Move mock cursor arrow pixel perfectly inside monitor boundaries
        mockCursor.style.left = `${mouseX}px`;
        mockCursor.style.top = `${mouseY}px`;
    });

    // 3. Command Box Copy to Clipboard Action
    const copyBtn = document.getElementById('copy-cmd-btn');
    const cmdText = document.getElementById('qemu-cmd');

    copyBtn.addEventListener('click', () => {
        navigator.clipboard.writeText(cmdText.innerText).then(() => {
            const originalHTML = copyBtn.innerHTML;
            copyBtn.innerHTML = `<i class="fa-solid fa-check"></i> Copied!`;
            copyBtn.style.background = 'hsl(174, 100%, 41%)';
            copyBtn.style.color = '#000';
            
            setTimeout(() => {
                copyBtn.innerHTML = originalHTML;
                copyBtn.style.background = '';
                copyBtn.style.color = '';
            }, 2000);
        }).catch(err => {
            console.error("Clipboard copy failed: ", err);
        });
    });

    // 4. Download Kernel Binary Binding
    const downloadBinBtn = document.getElementById('download-bin-btn');
    downloadBinBtn.addEventListener('click', (e) => {
        // Point download action directly to compiled bootable image in build targets folder
        downloadBinBtn.href = '../uloos-kernel/target/x86_64-unknown-none/debug/bootimage-uloos-kernel.bin';
        downloadBinBtn.setAttribute('download', 'bootimage-uloos-kernel.bin');
    });
});
