<div class="absolute inset-x-0 h-svh flex items-center justify-center overflow-hidden">
  <div class="ball border-union-accent-500">
    <span></span>
    <span></span>
    <span></span>
    <span></span>
    <span></span>
    <span></span>
  </div>
</div>

<style>
    @keyframes rotate {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }


    .ball span{
        width: 0;
        height: 0;
        border-radius: 50%;
        display: block;
        transition: all 2s ease-in-out;
        transform:translate(-50%,-50%);
        position:absolute;
        border: 1px solid transparent;
        animation: ballsAnimationBigger 6s infinite linear;
    }

    @keyframes ballsAnimationBigger {
        0% {
            border-color: transparent;
            opacity:0;
        }
        10% {
            border-color: inherit;
        }
        80%{
            border-color: transparent;
            opacity:1;
        }
        100% {
            width: 250vh;
            height: 250vh;
        }
    }

    .ball span:nth-child(2){
        animation-delay: 1s;
    }
    .ball span:nth-child(3){
        animation-delay: 2s;
    }
    .ball span:nth-child(4){
        animation-delay: 3s;
    }
    .ball span:nth-child(5){
        animation-delay: 4s;
    }
    .ball span:nth-child(6){
        animation-delay: 5s;
    }
</style>