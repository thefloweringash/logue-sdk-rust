__attribute__((section(".data.wavesA")))
void *wavesA;
__attribute__((section(".data.wavesB")))
void *wavesB;
__attribute__((section(".data.wavesC")))
void *wavesC;
__attribute__((section(".data.wavesD")))
void *wavesD;
__attribute__((section(".data.wavesE")))
void *wavesE;
__attribute__((section(".data.wavesF")))
void *wavesF;

__attribute__((section(".text._osc_rand")))
float _osc_rand()  { }

__attribute__((section(".text._osc_white")))
float _osc_white()  { }
