export default function useTheme(){
    function applyTheme(color){
        const root = document.documentElement;
        switch (color){
            case ("Black"):
            root.style.setProperty("--bg", "#000000");
            root.style.setProperty("--bg-secondary", "#1c1c1e");
            root.style.setProperty("--bg-tertiary", "#2c2c2e");
            root.style.setProperty("--text", "#FFFFFF");
            break;
            case ("Blue"):
            root.style.setProperty("--bg", "#00094f");
            root.style.setProperty("--bg-secondary", "#05116f");
            root.style.setProperty("--bg-tertiary", "#0d21b9");
            root.style.setProperty("--text", "#FFFFFF");
            break;
            case ("White"):
            root.style.setProperty("--bg", "#FFFFFF");
            root.style.setProperty("--bg-secondary", "#e2e2e2");
            root.style.setProperty("--bg-tertiary", "#acacac");
            root.style.setProperty("--text", "#000000");
            break;
            default:
            root.style.setProperty("--bg", "#000000");
            root.style.setProperty("--bg-secondary", "#1c1c1e");
            root.style.setProperty("--bg-tertiary", "#2c2c2e");
            root.style.setProperty("--text", "#FFFFFF");
            break;
        }
    }
    return { applyTheme }
}