<template>
    <div v-if="files.length > 0" class="file-previews">
        <div v-for="(file, index) in files" :key="index" class="file-preview">
            <img v-if="file.type.startsWith('image/')" :src="file.preview" alt="Preview" class="preview-image" />
            <div v-else class="file-document">
                <v-icon :icon="getFileIcon(file)" size="large" />
                <span class="file-name">{{ file.name }}</span>
            </div>
            <v-btn @click.stop="removeFile(index)" icon="mdi-close" size="x-small" variant="text" class="file-close" />
        </div>
    </div>
</template>

<script setup>
const props = defineProps({
    files: {
        type: Array,
        required: true,
    },
});

const emit = defineEmits(['remove-file']);

const removeFile = (index) => {
    emit('remove-file', index);
};

const getFileIcon = (file) => {
    const extension = file.name.split('.').pop().toLowerCase();

    const icons = {
        pdf: 'mdi-file-pdf-box',
        doc: 'mdi-file-word',
        docx: 'mdi-file-word',
        odt: 'mdi-file-word',
        rtf: 'mdi-file-word',
        tex: 'mdi-file-word',

        xls: 'mdi-file-excel',
        xlsx: 'mdi-file-excel',
        ods: 'mdi-file-excel',
        csv: 'mdi-file-delimited',

        ppt: 'mdi-file-powerpoint',
        pptx: 'mdi-file-powerpoint',
        odp: 'mdi-file-powerpoint',

        zip: 'mdi-folder-zip',
        rar: 'mdi-folder-zip',
        '7z': 'mdi-folder-zip',
        tar: 'mdi-folder-zip',
        gz: 'mdi-folder-zip',

        txt: 'mdi-note-text',
        md: 'mdi-language-markdown',
        log: 'mdi-text-box',

        js: 'mdi-language-javascript',
        ts: 'mdi-language-typescript',
        py: 'mdi-language-python',
        java: 'mdi-language-java',
        rb: 'mdi-language-ruby',
        php: 'mdi-language-php',
        c: 'mdi-language-c',
        cpp: 'mdi-language-cpp',
        cs: 'mdi-language-csharp',
        go: 'mdi-language-go',
        swift: 'mdi-language-swift',
        kotlin: 'mdi-language-kotlin',
        rust: 'mdi-language-rust',

        html: 'mdi-language-html5',
        css: 'mdi-language-css3',
        scss: 'mdi-sass',
        less: 'mdi-less-than',
        json: 'mdi-code-json',
        xml: 'mdi-xml',
        yml: 'mdi-yaml',
        yaml: 'mdi-yaml',

        sql: 'mdi-database',
        db: 'mdi-database',
        mdb: 'mdi-database',

        jpg: 'mdi-image',
        jpeg: 'mdi-image',
        png: 'mdi-image',
        svg: 'mdi-image',

        ini: 'mdi-cog',
        cfg: 'mdi-cog',
        psd: 'mdi-image-edit',

    };
    return icons[extension] || 'mdi-file';
};
</script>

<style scoped>
.file-previews {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    flex-wrap: wrap;
}

.file-preview {
    position: relative;
    width: 80px;
    height: 80px;
    border-radius: 4px;
    background-color: #2d2d2d;
    overflow: visible;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
}

.preview-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 4px;
}

.file-document {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    width: 100%;
    padding: 8px;
    color: #b0b0b0;
    font-size: 12px;
    text-align: center;
    word-break: break-word;
}

.file-document .v-icon {
    margin-bottom: 4px;
    color: #90a4ae;
}

.file-name {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
}

.file-close {
    position: absolute;
    top: -12px;
    right: -12px;
    background-color: #424242;
    border-radius: 50%;
    color: white !important;
    opacity: 0.9;
    transform: scale(0.8);
    transition: all 0.2s;
    z-index: 2;
}

.file-close:hover {
    opacity: 1;
    background-color: #b90000;
    transform: scale(1);
}
</style>